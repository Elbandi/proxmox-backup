use std::os::unix::prelude::OsStrExt;
use std::process::Command;

use anyhow::{bail, format_err, Error};
use serde_json::Value;

use proxmox_sys::linux::procfs;

use proxmox_router::{ApiMethod, Permission, Router, RpcEnvironment};
use proxmox_schema::api;

use pbs_api_types::{
    NodePowerCommand, StorageStatus, NODE_SCHEMA, PRIV_SYS_AUDIT, PRIV_SYS_POWER_MANAGEMENT,
};

use pbs_api_types::{
    NodeCpuInformation, NodeInformation, NodeMemoryCounters, NodeStatus, NodeSwapCounters,
};

fn procfs_to_node_cpu_info(info: procfs::ProcFsCPUInfo) -> NodeCpuInformation {
    NodeCpuInformation {
        model: info.model,
        sockets: info.sockets,
        cpus: info.cpus,
    }
}

#[api(
    input: {
        properties: {
            node: {
                schema: NODE_SCHEMA,
            },
        },
    },
    returns: {
        type: NodeStatus,
    },
    access: {
        permission: &Permission::Privilege(&["system", "status"], PRIV_SYS_AUDIT, false),
    },
)]
/// Read node memory, CPU and (root) disk usage
async fn get_status(
    _param: Value,
    _info: &ApiMethod,
    _rpcenv: &mut dyn RpcEnvironment,
) -> Result<NodeStatus, Error> {
    let meminfo: procfs::ProcFsMemInfo = procfs::read_meminfo()?;
    let memory = NodeMemoryCounters {
        total: meminfo.memtotal,
        used: meminfo.memused,
        free: meminfo.memfree,
    };

    let swap = NodeSwapCounters {
        total: meminfo.swaptotal,
        used: meminfo.swapused,
        free: meminfo.swapfree,
    };

    let kstat: procfs::ProcFsStat = procfs::read_proc_stat()?;
    let cpu = kstat.cpu;
    let wait = kstat.iowait_percent;

    let loadavg = procfs::Loadavg::read()?;
    let loadavg = [loadavg.one(), loadavg.five(), loadavg.fifteen()];

    let cpuinfo = procfs::read_cpuinfo()?;
    let cpuinfo = procfs_to_node_cpu_info(cpuinfo);

    let uname = nix::sys::utsname::uname()?;
    let kversion = format!(
        "{} {} {}",
        std::str::from_utf8(uname.sysname().as_bytes())?,
        std::str::from_utf8(uname.release().as_bytes())?,
        std::str::from_utf8(uname.version().as_bytes())?
    );

    let disk = crate::tools::fs::fs_info_static(proxmox_lang::c_str!("/")).await?;

    Ok(NodeStatus {
        memory,
        swap,
        root: StorageStatus {
            total: disk.total,
            used: disk.used,
            avail: disk.available,
        },
        uptime: procfs::read_proc_uptime()?.0 as u64,
        loadavg,
        kversion,
        cpuinfo,
        cpu,
        wait,
        info: NodeInformation {
            fingerprint: crate::cert_info()?.fingerprint()?,
        },
    })
}

#[api(
    protected: true,
    input: {
        properties: {
            node: {
                schema: NODE_SCHEMA,
            },
            command: {
                type: NodePowerCommand,
            },
        }
    },
    access: {
        permission: &Permission::Privilege(&["system", "status"], PRIV_SYS_POWER_MANAGEMENT, false),
    },
)]
/// Reboot or shutdown the node.
fn reboot_or_shutdown(command: NodePowerCommand) -> Result<(), Error> {
    let systemctl_command = match command {
        NodePowerCommand::Reboot => "reboot",
        NodePowerCommand::Shutdown => "poweroff",
    };

    let output = Command::new("systemctl")
        .arg(systemctl_command)
        .output()
        .map_err(|err| format_err!("failed to execute systemctl - {}", err))?;

    if !output.status.success() {
        match output.status.code() {
            Some(code) => {
                let msg = String::from_utf8(output.stderr)
                    .map(|m| {
                        if m.is_empty() {
                            String::from("no error message")
                        } else {
                            m
                        }
                    })
                    .unwrap_or_else(|_| String::from("non utf8 error message (suppressed)"));
                bail!("diff failed with status code: {} - {}", code, msg);
            }
            None => bail!("systemctl terminated by signal"),
        }
    }
    Ok(())
}

pub const ROUTER: Router = Router::new()
    .get(&API_METHOD_GET_STATUS)
    .post(&API_METHOD_REBOOT_OR_SHUTDOWN);
