use apt_pkg_native::Cache;
use anyhow::{Error, bail};
use serde_json::{json, Value};

use proxmox::{list_subdirs_api_method, const_regex};
use proxmox::api::{api, RpcEnvironment, RpcEnvironmentType, Permission};
use proxmox::api::router::{Router, SubdirMap};

use crate::server::WorkerTask;

use crate::config::acl::{PRIV_SYS_AUDIT, PRIV_SYS_MODIFY};
use crate::api2::types::{APTUpdateInfo, NODE_SCHEMA, Userid, UPID_SCHEMA};

const_regex! {
    VERSION_EPOCH_REGEX = r"^\d+:";
    FILENAME_EXTRACT_REGEX = r"^.*/.*?_(.*)_Packages$";
}

// FIXME: Replace with call to 'apt changelog <pkg> --print-uris'. Currently
// not possible as our packages do not have a URI set in their Release file
fn get_changelog_url(
    package: &str,
    filename: &str,
    source_pkg: &str,
    version: &str,
    source_version: &str,
    origin: &str,
    component: &str,
) -> Result<String, Error> {
    if origin == "" {
        bail!("no origin available for package {}", package);
    }

    if origin == "Debian" {
        let source_version = (VERSION_EPOCH_REGEX.regex_obj)().replace_all(source_version, "");

        let prefix = if source_pkg.starts_with("lib") {
            source_pkg.get(0..4)
        } else {
            source_pkg.get(0..1)
        };

        let prefix = match prefix {
            Some(p) => p,
            None => bail!("cannot get starting characters of package name '{}'", package)
        };

        // note: security updates seem to not always upload a changelog for
        // their package version, so this only works *most* of the time
        return Ok(format!("https://metadata.ftp-master.debian.org/changelogs/main/{}/{}/{}_{}_changelog",
                          prefix, source_pkg, source_pkg, source_version));

    } else if origin == "Proxmox" {
        let version = (VERSION_EPOCH_REGEX.regex_obj)().replace_all(version, "");

        let base = match (FILENAME_EXTRACT_REGEX.regex_obj)().captures(filename) {
            Some(captures) => {
                let base_capture = captures.get(1);
                match base_capture {
                    Some(base_underscore) => base_underscore.as_str().replace("_", "/"),
                    None => bail!("incompatible filename, cannot find regex group")
                }
            },
            None => bail!("incompatible filename, doesn't match regex")
        };

        return Ok(format!("http://download.proxmox.com/{}/{}_{}.changelog",
                          base, package, version));
    }

    bail!("unknown origin ({}) or component ({})", origin, component)
}

struct FilterData<'a> {
    // this is version info returned by APT
    installed_version: &'a str,
    candidate_version: &'a str,

    // this is the version info the filter is supposed to check
    active_version: &'a str,
}

fn list_installed_apt_packages<F: Fn(FilterData) -> bool>(filter: F)
    -> Vec<APTUpdateInfo> {

    let mut ret = Vec::new();

    // note: this is not an 'apt update', it just re-reads the cache from disk
    let mut cache = Cache::get_singleton();
    cache.reload();

    let mut cache_iter = cache.iter();

    loop {
        let view = match cache_iter.next() {
            Some(view) => view,
            None => break
        };

        let current_version = view.current_version();
        let candidate_version = view.candidate_version();

        let (current_version, candidate_version) = match (current_version, candidate_version) {
            (Some(cur), Some(can)) => (cur, can), // package installed and there is an update
            (Some(cur), None) => (cur.clone(), cur), // package installed and up-to-date
            (None, Some(_)) => continue, // package could be installed
            (None, None) => continue, // broken
        };

        // get additional information via nested APT 'iterators'
        let mut view_iter = view.versions();
        while let Some(ver) = view_iter.next() {

            let package = view.name();
            let version = ver.version();
            let mut origin_res = "unknown".to_owned();
            let mut section_res = "unknown".to_owned();
            let mut priority_res = "unknown".to_owned();
            let mut change_log_url = "".to_owned();
            let mut short_desc = package.clone();
            let mut long_desc = "".to_owned();

            let fd = FilterData {
                installed_version: &current_version,
                candidate_version: &candidate_version,
                active_version: &version,
            };

            if filter(fd) {
                if let Some(section) = ver.section() {
                    section_res = section;
                }

                if let Some(prio) = ver.priority_type() {
                    priority_res = prio;
                }

                // assume every package has only one origin file (not
                // origin, but origin *file*, for some reason those seem to
                // be different concepts in APT)
                let mut origin_iter = ver.origin_iter();
                let origin = origin_iter.next();
                if let Some(origin) = origin {

                    if let Some(sd) = origin.short_desc() {
                        short_desc = sd;
                    }

                    if let Some(ld) = origin.long_desc() {
                        long_desc = ld;
                    }

                    // the package files appear in priority order, meaning
                    // the one for the candidate version is first - this is fine
                    // however, as the source package should be the same for all
                    // versions anyway
                    let mut pkg_iter = origin.file();
                    let pkg_file = pkg_iter.next();
                    if let Some(pkg_file) = pkg_file {
                        if let Some(origin_name) = pkg_file.origin() {
                            origin_res = origin_name;
                        }

                        let filename = pkg_file.file_name();
                        let source_pkg = ver.source_package();
                        let source_ver = ver.source_version();
                        let component = pkg_file.component();

                        // build changelog URL from gathered information
                        // ignore errors, use empty changelog instead
                        let url = get_changelog_url(&package, &filename, &source_pkg,
                            &version, &source_ver, &origin_res, &component);
                        if let Ok(url) = url {
                            change_log_url = url;
                        }
                    }
                }

                let info = APTUpdateInfo {
                    package,
                    title: short_desc,
                    arch: view.arch(),
                    description: long_desc,
                    change_log_url,
                    origin: origin_res,
                    version: candidate_version.clone(),
                    old_version: current_version.clone(),
                    priority: priority_res,
                    section: section_res,
                };
                ret.push(info);
            }
        }
    }

    return ret;
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
        description: "A list of packages with available updates.",
        type: Array,
        items: { type: APTUpdateInfo },
    },
    access: {
        permission: &Permission::Privilege(&[], PRIV_SYS_AUDIT, false),
    },
)]
/// List available APT updates
fn apt_update_available(_param: Value) -> Result<Value, Error> {
    let all_upgradeable = list_installed_apt_packages(|data|
        data.candidate_version == data.active_version &&
        data.installed_version != data.candidate_version
    );
    Ok(json!(all_upgradeable))
}

#[api(
    protected: true,
    input: {
        properties: {
            node: {
                schema: NODE_SCHEMA,
            },
            quiet: {
                description: "Only produces output suitable for logging, omitting progress indicators.",
                type: bool,
                default: false,
                optional: true,
            },
        },
    },
    returns: {
        schema: UPID_SCHEMA,
    },
    access: {
        permission: &Permission::Privilege(&[], PRIV_SYS_MODIFY, false),
    },
)]
/// Update the APT database
pub fn apt_update_database(
    quiet: Option<bool>,
    rpcenv: &mut dyn RpcEnvironment,
) -> Result<String, Error> {

    let userid: Userid = rpcenv.get_user().unwrap().parse()?;
    let to_stdout = if rpcenv.env_type() == RpcEnvironmentType::CLI { true } else { false };
    let quiet = quiet.unwrap_or(API_METHOD_APT_UPDATE_DATABASE_PARAM_DEFAULT_QUIET);

    let upid_str = WorkerTask::new_thread("aptupdate", None, userid, to_stdout, move |worker| {
        if !quiet { worker.log("starting apt-get update") }

        // TODO: set proxy /etc/apt/apt.conf.d/76pbsproxy like PVE

        let mut command = std::process::Command::new("apt-get");
        command.arg("update");

        let output = crate::tools::run_command(command, None)?;
        if !quiet { worker.log(output) }

        // TODO: add mail notify for new updates like PVE

        Ok(())
    })?;

    Ok(upid_str)
}

const SUBDIRS: SubdirMap = &[
    ("update", &Router::new()
        .get(&API_METHOD_APT_UPDATE_AVAILABLE)
        .post(&API_METHOD_APT_UPDATE_DATABASE)
    ),
];

pub const ROUTER: Router = Router::new()
    .get(&list_subdirs_api_method!(SUBDIRS))
    .subdirs(SUBDIRS);
