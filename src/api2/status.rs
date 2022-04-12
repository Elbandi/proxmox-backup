//! Datastote status

use anyhow::Error;
use serde_json::Value;

use proxmox_schema::api;
use proxmox_router::{
    ApiMethod,
    Permission,
    Router,
    RpcEnvironment,
    SubdirMap,
};
use proxmox_router::list_subdirs_api_method;

use pbs_api_types::{
    Authid, DataStoreStatusListItem, Operation, RRDMode, RRDTimeFrame,
    PRIV_DATASTORE_AUDIT, PRIV_DATASTORE_BACKUP,
};

use pbs_datastore::DataStore;
use pbs_config::CachedUserInfo;

use crate::tools::statistics::{linear_regression};
use crate::rrd_cache::extract_rrd_data;

#[api(
    returns: {
        description: "Lists the Status of the Datastores.",
        type: Array,
        items: {
            type: DataStoreStatusListItem,
        },
    },
    access: {
        permission: &Permission::Anybody,
    },
)]
/// List Datastore usages and estimates
pub fn datastore_status(
    _param: Value,
    _info: &ApiMethod,
    rpcenv: &mut dyn RpcEnvironment,
    ) -> Result<Vec<DataStoreStatusListItem>, Error> {

    let (config, _digest) = pbs_config::datastore::config()?;

    let auth_id: Authid = rpcenv.get_auth_id().unwrap().parse()?;
    let user_info = CachedUserInfo::new()?;

    let mut list = Vec::new();

    for (store, (_, _)) in &config.sections {
        let user_privs = user_info.lookup_privs(&auth_id, &["datastore", store]);
        let allowed = (user_privs & (PRIV_DATASTORE_AUDIT| PRIV_DATASTORE_BACKUP)) != 0;
        if !allowed {
            continue;
        }

        let datastore = match DataStore::lookup_datastore(&store, Some(Operation::Read)) {
            Ok(datastore) => datastore,
            Err(err) => {
                list.push(DataStoreStatusListItem {
                    store: store.clone(),
                    total: -1,
                    used: -1,
                    avail: -1,
                    history: None,
                    history_start: None,
                    history_delta: None,
                    estimated_full_date: None,
                    error: Some(err.to_string()),
                });
                continue;
            }
        };
        let status = crate::tools::disks::disk_usage(&datastore.base_path())?;

        let mut entry = DataStoreStatusListItem {
            store: store.clone(),
            total: status.total as i64,
            used: status.used as i64,
            avail: status.avail as i64,
            history: None,
            history_start: None,
            history_delta: None,
            estimated_full_date: None,
            error: None,
        };

        let rrd_dir = format!("datastore/{}", store);

        let get_rrd = |what: &str| extract_rrd_data(
            &rrd_dir,
            what,
            RRDTimeFrame::Month,
            RRDMode::Average,
        );

        let total_res = get_rrd("total")?;
        let used_res = get_rrd("used")?;

        if let (Some((start, reso, total_list)), Some((_, _, used_list))) = (total_res, used_res) {
            let mut usage_list: Vec<f64> = Vec::new();
            let mut time_list: Vec<u64> = Vec::new();
            let mut history = Vec::new();

            for (idx, used) in used_list.iter().enumerate() {
                let total = if idx < total_list.len() {
                    total_list[idx]
                } else {
                    None
                };

                match (total, used) {
                    (Some(total), Some(used)) if total != 0.0 => {
                        time_list.push(start + (idx as u64)*reso);
                        let usage = used/total;
                        usage_list.push(usage);
                        history.push(Some(usage));
                    },
                    _ => {
                        history.push(None)
                    }
                }
            }

            entry.history_start = Some(start);
            entry.history_delta = Some(reso);
            entry.history = Some(history);

            // we skip the calculation for datastores with not enough data
            if usage_list.len() >= 7 {
                entry.estimated_full_date = match linear_regression(&time_list, &usage_list) {
                    Some((a, b)) if b != 0.0 => Some(((1.0 - a) / b).floor() as i64),
                    Some((_, b)) if b == 0.0 => Some(0), // infinite estimate, set to past for gui to detect
                    _ => None,
                };
            }
        }

        list.push(entry);
    }

    Ok(list.into())
}

const SUBDIRS: SubdirMap = &[
    ("datastore-usage", &Router::new().get(&API_METHOD_DATASTORE_STATUS)),
];

pub const ROUTER: Router = Router::new()
    .get(&list_subdirs_api_method!(SUBDIRS))
    .subdirs(SUBDIRS);
