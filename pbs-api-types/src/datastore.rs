use serde::{Deserialize, Serialize};

use proxmox_schema::{
    api, const_regex, ApiStringFormat, ApiType, ArraySchema, EnumEntry, IntegerSchema, ReturnType,
    Schema, StringSchema, Updater,
};

use crate::{
    Authid, CryptMode, Fingerprint, MaintenanceMode, Userid, DATASTORE_NOTIFY_STRING_SCHEMA,
    GC_SCHEDULE_SCHEMA, PROXMOX_SAFE_ID_FORMAT, PRUNE_SCHEDULE_SCHEMA, SHA256_HEX_REGEX,
    SINGLE_LINE_COMMENT_SCHEMA, UPID,
};

const_regex! {
    pub BACKUP_TYPE_REGEX = concat!(r"^(", BACKUP_TYPE_RE!(), r")$");

    pub BACKUP_ID_REGEX = concat!(r"^", BACKUP_ID_RE!(), r"$");

    pub BACKUP_DATE_REGEX = concat!(r"^", BACKUP_TIME_RE!() ,r"$");

    pub GROUP_PATH_REGEX = concat!(r"^(", BACKUP_TYPE_RE!(), ")/(", BACKUP_ID_RE!(), r")$");

    pub BACKUP_FILE_REGEX = r"^.*\.([fd]idx|blob)$";

    pub SNAPSHOT_PATH_REGEX = concat!(r"^", SNAPSHOT_PATH_REGEX_STR!(), r"$");

    pub DATASTORE_MAP_REGEX = concat!(r"(:?", PROXMOX_SAFE_ID_REGEX_STR!(), r"=)?", PROXMOX_SAFE_ID_REGEX_STR!());
}

pub const CHUNK_DIGEST_FORMAT: ApiStringFormat = ApiStringFormat::Pattern(&SHA256_HEX_REGEX);

pub const DIR_NAME_SCHEMA: Schema = StringSchema::new("Directory name")
    .min_length(1)
    .max_length(4096)
    .schema();

pub const BACKUP_ARCHIVE_NAME_SCHEMA: Schema = StringSchema::new("Backup archive name.")
    .format(&PROXMOX_SAFE_ID_FORMAT)
    .schema();

pub const BACKUP_ID_FORMAT: ApiStringFormat = ApiStringFormat::Pattern(&BACKUP_ID_REGEX);
pub const BACKUP_GROUP_FORMAT: ApiStringFormat = ApiStringFormat::Pattern(&GROUP_PATH_REGEX);

pub const BACKUP_ID_SCHEMA: Schema = StringSchema::new("Backup ID.")
    .format(&BACKUP_ID_FORMAT)
    .schema();

pub const BACKUP_TYPE_SCHEMA: Schema = StringSchema::new("Backup type.")
    .format(&ApiStringFormat::Enum(&[
        EnumEntry::new("vm", "Virtual Machine Backup"),
        EnumEntry::new("ct", "Container Backup"),
        EnumEntry::new("host", "Host Backup"),
    ]))
    .schema();

pub const BACKUP_TIME_SCHEMA: Schema = IntegerSchema::new("Backup time (Unix epoch.)")
    .minimum(1)
    .schema();

pub const BACKUP_GROUP_SCHEMA: Schema = StringSchema::new("Backup Group")
    .format(&BACKUP_GROUP_FORMAT)
    .schema();

pub const DATASTORE_SCHEMA: Schema = StringSchema::new("Datastore name.")
    .format(&PROXMOX_SAFE_ID_FORMAT)
    .min_length(3)
    .max_length(32)
    .schema();

pub const CHUNK_DIGEST_SCHEMA: Schema = StringSchema::new("Chunk digest (SHA256).")
    .format(&CHUNK_DIGEST_FORMAT)
    .schema();

pub const DATASTORE_MAP_FORMAT: ApiStringFormat = ApiStringFormat::Pattern(&DATASTORE_MAP_REGEX);

pub const DATASTORE_MAP_SCHEMA: Schema = StringSchema::new("Datastore mapping.")
    .format(&DATASTORE_MAP_FORMAT)
    .min_length(3)
    .max_length(65)
    .type_text("(<source>=)?<target>")
    .schema();

pub const DATASTORE_MAP_ARRAY_SCHEMA: Schema =
    ArraySchema::new("Datastore mapping list.", &DATASTORE_MAP_SCHEMA).schema();

pub const DATASTORE_MAP_LIST_SCHEMA: Schema = StringSchema::new(
    "A list of Datastore mappings (or single datastore), comma separated. \
    For example 'a=b,e' maps the source datastore 'a' to target 'b and \
    all other sources to the default 'e'. If no default is given, only the \
    specified sources are mapped.",
)
.format(&ApiStringFormat::PropertyString(
    &DATASTORE_MAP_ARRAY_SCHEMA,
))
.schema();

pub const PRUNE_SCHEMA_KEEP_DAILY: Schema = IntegerSchema::new("Number of daily backups to keep.")
    .minimum(1)
    .schema();

pub const PRUNE_SCHEMA_KEEP_HOURLY: Schema =
    IntegerSchema::new("Number of hourly backups to keep.")
        .minimum(1)
        .schema();

pub const PRUNE_SCHEMA_KEEP_LAST: Schema = IntegerSchema::new("Number of backups to keep.")
    .minimum(1)
    .schema();

pub const PRUNE_SCHEMA_KEEP_MONTHLY: Schema =
    IntegerSchema::new("Number of monthly backups to keep.")
        .minimum(1)
        .schema();

pub const PRUNE_SCHEMA_KEEP_WEEKLY: Schema =
    IntegerSchema::new("Number of weekly backups to keep.")
        .minimum(1)
        .schema();

pub const PRUNE_SCHEMA_KEEP_YEARLY: Schema =
    IntegerSchema::new("Number of yearly backups to keep.")
        .minimum(1)
        .schema();

#[api(
    properties: {
        "keep-last": {
            schema: PRUNE_SCHEMA_KEEP_LAST,
            optional: true,
        },
        "keep-hourly": {
            schema: PRUNE_SCHEMA_KEEP_HOURLY,
            optional: true,
        },
        "keep-daily": {
            schema: PRUNE_SCHEMA_KEEP_DAILY,
            optional: true,
        },
        "keep-weekly": {
            schema: PRUNE_SCHEMA_KEEP_WEEKLY,
            optional: true,
        },
        "keep-monthly": {
            schema: PRUNE_SCHEMA_KEEP_MONTHLY,
            optional: true,
        },
        "keep-yearly": {
            schema: PRUNE_SCHEMA_KEEP_YEARLY,
            optional: true,
        },
    }
)]
#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
/// Common pruning options
pub struct PruneOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keep_last: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keep_hourly: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keep_daily: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keep_weekly: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keep_monthly: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keep_yearly: Option<u64>,
}

#[api]
#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
/// The order to sort chunks by
pub enum ChunkOrder {
    /// Iterate chunks in the index order
    None,
    /// Iterate chunks in inode order
    Inode,
}

#[api(
    properties: {
        "chunk-order": {
            type: ChunkOrder,
            optional: true,
        },
    },
)]
#[derive(Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
/// Datastore tuning options
pub struct DatastoreTuning {
    /// Iterate chunks in this order
    pub chunk_order: Option<ChunkOrder>,
}

pub const DATASTORE_TUNING_STRING_SCHEMA: Schema = StringSchema::new("Datastore tuning options")
    .format(&ApiStringFormat::PropertyString(
        &DatastoreTuning::API_SCHEMA,
    ))
    .schema();

#[api(
    properties: {
        name: {
            schema: DATASTORE_SCHEMA,
        },
        path: {
            schema: DIR_NAME_SCHEMA,
        },
        "notify-user": {
            optional: true,
            type: Userid,
        },
        "notify": {
            optional: true,
            schema: DATASTORE_NOTIFY_STRING_SCHEMA,
        },
        comment: {
            optional: true,
            schema: SINGLE_LINE_COMMENT_SCHEMA,
        },
        "gc-schedule": {
            optional: true,
            schema: GC_SCHEDULE_SCHEMA,
        },
        "prune-schedule": {
            optional: true,
            schema: PRUNE_SCHEDULE_SCHEMA,
        },
        "keep-last": {
            optional: true,
            schema: PRUNE_SCHEMA_KEEP_LAST,
        },
        "keep-hourly": {
            optional: true,
            schema: PRUNE_SCHEMA_KEEP_HOURLY,
        },
        "keep-daily": {
            optional: true,
            schema: PRUNE_SCHEMA_KEEP_DAILY,
        },
        "keep-weekly": {
            optional: true,
            schema: PRUNE_SCHEMA_KEEP_WEEKLY,
        },
        "keep-monthly": {
            optional: true,
            schema: PRUNE_SCHEMA_KEEP_MONTHLY,
        },
        "keep-yearly": {
            optional: true,
            schema: PRUNE_SCHEMA_KEEP_YEARLY,
        },
        "verify-new": {
            description: "If enabled, all new backups will be verified right after completion.",
            optional: true,
            type: bool,
        },
        tuning: {
            optional: true,
            schema: DATASTORE_TUNING_STRING_SCHEMA,
        },
        "maintenance-mode": {
            optional: true,
            format: &ApiStringFormat::PropertyString(&MaintenanceMode::API_SCHEMA),
            type: String,
        },
    }
)]
#[derive(Serialize, Deserialize, Updater)]
#[serde(rename_all = "kebab-case")]
/// Datastore configuration properties.
pub struct DataStoreConfig {
    #[updater(skip)]
    pub name: String,
    #[updater(skip)]
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gc_schedule: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prune_schedule: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keep_last: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keep_hourly: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keep_daily: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keep_weekly: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keep_monthly: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keep_yearly: Option<u64>,
    /// If enabled, all backups will be verified right after completion.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verify_new: Option<bool>,
    /// Send job email notification to this user
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notify_user: Option<Userid>,
    /// Send notification only for job errors
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notify: Option<String>,
    /// Datastore tuning options
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tuning: Option<String>,
    /// Maintenance mode, type is either 'offline' or 'read-only', message should be enclosed in "
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maintenance_mode: Option<String>,
}

impl DataStoreConfig {
    pub fn get_maintenance_mode(&self) -> Option<MaintenanceMode> {
        self.maintenance_mode
            .as_ref()
            .and_then(|str| MaintenanceMode::API_SCHEMA.parse_property_string(str).ok())
            .and_then(|value| MaintenanceMode::deserialize(value).ok())
    }
}

#[api(
    properties: {
        store: {
            schema: DATASTORE_SCHEMA,
        },
        comment: {
            optional: true,
            schema: SINGLE_LINE_COMMENT_SCHEMA,
        },
    },
)]
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
/// Basic information about a datastore.
pub struct DataStoreListItem {
    pub store: String,
    pub comment: Option<String>,
}

#[api(
    properties: {
        "filename": {
            schema: BACKUP_ARCHIVE_NAME_SCHEMA,
        },
        "crypt-mode": {
            type: CryptMode,
            optional: true,
        },
    },
)]
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
/// Basic information about archive files inside a backup snapshot.
pub struct BackupContent {
    pub filename: String,
    /// Info if file is encrypted, signed, or neither.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub crypt_mode: Option<CryptMode>,
    /// Archive size (from backup manifest).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<u64>,
}

#[api()]
#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
/// Result of a verify operation.
pub enum VerifyState {
    /// Verification was successful
    Ok,
    /// Verification reported one or more errors
    Failed,
}

#[api(
    properties: {
        upid: {
            type: UPID,
        },
        state: {
            type: VerifyState,
        },
    },
)]
#[derive(Serialize, Deserialize)]
/// Task properties.
pub struct SnapshotVerifyState {
    /// UPID of the verify task
    pub upid: UPID,
    /// State of the verification. Enum.
    pub state: VerifyState,
}

#[api(
    properties: {
        "backup-type": {
            schema: BACKUP_TYPE_SCHEMA,
        },
        "backup-id": {
            schema: BACKUP_ID_SCHEMA,
        },
        "backup-time": {
            schema: BACKUP_TIME_SCHEMA,
        },
        comment: {
            schema: SINGLE_LINE_COMMENT_SCHEMA,
            optional: true,
        },
        verification: {
            type: SnapshotVerifyState,
            optional: true,
        },
        fingerprint: {
            type: String,
            optional: true,
        },
        files: {
            items: {
                schema: BACKUP_ARCHIVE_NAME_SCHEMA
            },
        },
        owner: {
            type: Authid,
            optional: true,
        },
    },
)]
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
/// Basic information about backup snapshot.
pub struct SnapshotListItem {
    pub backup_type: String, // enum
    pub backup_id: String,
    pub backup_time: i64,
    /// The first line from manifest "notes"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    /// The result of the last run verify task
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verification: Option<SnapshotVerifyState>,
    /// Fingerprint of encryption key
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fingerprint: Option<Fingerprint>,
    /// List of contained archive files.
    pub files: Vec<BackupContent>,
    /// Overall snapshot size (sum of all archive sizes).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<u64>,
    /// The owner of the snapshots group
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<Authid>,
    /// Protection from prunes
    #[serde(default)]
    pub protected: bool,
}

#[api(
    properties: {
        "backup-type": {
            schema: BACKUP_TYPE_SCHEMA,
        },
        "backup-id": {
            schema: BACKUP_ID_SCHEMA,
        },
        "last-backup": {
            schema: BACKUP_TIME_SCHEMA,
        },
        "backup-count": {
            type: Integer,
        },
        files: {
            items: {
                schema: BACKUP_ARCHIVE_NAME_SCHEMA
            },
        },
        owner: {
            type: Authid,
            optional: true,
        },
    },
)]
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
/// Basic information about a backup group.
pub struct GroupListItem {
    pub backup_type: String, // enum
    pub backup_id: String,
    pub last_backup: i64,
    /// Number of contained snapshots
    pub backup_count: u64,
    /// List of contained archive files.
    pub files: Vec<String>,
    /// The owner of group
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<Authid>,
    /// The first line from group "notes"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
}

#[api(
    properties: {
        "backup-type": {
            schema: BACKUP_TYPE_SCHEMA,
        },
        "backup-id": {
            schema: BACKUP_ID_SCHEMA,
        },
        "backup-time": {
            schema: BACKUP_TIME_SCHEMA,
        },
    },
)]
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
/// Prune result.
pub struct PruneListItem {
    pub backup_type: String, // enum
    pub backup_id: String,
    pub backup_time: i64,
    /// Keep snapshot
    pub keep: bool,
}

#[api(
    properties: {
        ct: {
            type: TypeCounts,
            optional: true,
        },
        host: {
            type: TypeCounts,
            optional: true,
        },
        vm: {
            type: TypeCounts,
            optional: true,
        },
        other: {
            type: TypeCounts,
            optional: true,
        },
    },
)]
#[derive(Serialize, Deserialize, Default)]
/// Counts of groups/snapshots per BackupType.
pub struct Counts {
    /// The counts for CT backups
    pub ct: Option<TypeCounts>,
    /// The counts for Host backups
    pub host: Option<TypeCounts>,
    /// The counts for VM backups
    pub vm: Option<TypeCounts>,
    /// The counts for other backup types
    pub other: Option<TypeCounts>,
}

#[api()]
#[derive(Serialize, Deserialize, Default)]
/// Backup Type group/snapshot counts.
pub struct TypeCounts {
    /// The number of groups of the type.
    pub groups: u64,
    /// The number of snapshots of the type.
    pub snapshots: u64,
}

#[api(
    properties: {
        "upid": {
            optional: true,
            type: UPID,
        },
    },
)]
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
/// Garbage collection status.
pub struct GarbageCollectionStatus {
    pub upid: Option<String>,
    /// Number of processed index files.
    pub index_file_count: usize,
    /// Sum of bytes referred by index files.
    pub index_data_bytes: u64,
    /// Bytes used on disk.
    pub disk_bytes: u64,
    /// Chunks used on disk.
    pub disk_chunks: usize,
    /// Sum of removed bytes.
    pub removed_bytes: u64,
    /// Number of removed chunks.
    pub removed_chunks: usize,
    /// Sum of pending bytes (pending removal - kept for safety).
    pub pending_bytes: u64,
    /// Number of pending chunks (pending removal - kept for safety).
    pub pending_chunks: usize,
    /// Number of chunks marked as .bad by verify that have been removed by GC.
    pub removed_bad: usize,
    /// Number of chunks still marked as .bad after garbage collection.
    pub still_bad: usize,
}

impl Default for GarbageCollectionStatus {
    fn default() -> Self {
        GarbageCollectionStatus {
            upid: None,
            index_file_count: 0,
            index_data_bytes: 0,
            disk_bytes: 0,
            disk_chunks: 0,
            removed_bytes: 0,
            removed_chunks: 0,
            pending_bytes: 0,
            pending_chunks: 0,
            removed_bad: 0,
            still_bad: 0,
        }
    }
}

#[api(
    properties: {
        "gc-status": {
            type: GarbageCollectionStatus,
            optional: true,
        },
        counts: {
            type: Counts,
            optional: true,
        },
    },
)]
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
/// Overall Datastore status and useful information.
pub struct DataStoreStatus {
    /// Total space (bytes).
    pub total: u64,
    /// Used space (bytes).
    pub used: u64,
    /// Available space (bytes).
    pub avail: u64,
    /// Status of last GC
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gc_status: Option<GarbageCollectionStatus>,
    /// Group/Snapshot counts
    #[serde(skip_serializing_if = "Option::is_none")]
    pub counts: Option<Counts>,
}

#[api(
    properties: {
        store: {
            schema: DATASTORE_SCHEMA,
        },
        history: {
            type: Array,
            optional: true,
            items: {
                type: Number,
                description: "The usage of a time in the past. Either null or between 0.0 and 1.0.",
            }
        },
     },
)]
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
/// Status of a Datastore
pub struct DataStoreStatusListItem {
    pub store: String,
    /// The Size of the underlying storage in bytes. (-1 on error)
    pub total: i64,
    /// The used bytes of the underlying storage. (-1 on error)
    pub used: i64,
    /// The available bytes of the underlying storage. (-1 on error)
    pub avail: i64,
    /// A list of usages of the past (last Month).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub history: Option<Vec<Option<f64>>>,
    /// History start time (epoch)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub history_start: Option<u64>,
    /// History resolution (seconds)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub history_delta: Option<u64>,
    /// Estimation of the UNIX epoch when the storage will be full.
    /// This is calculated via a simple Linear Regression (Least
    /// Squares) of RRD data of the last Month. Missing if there are
    /// not enough data points yet. If the estimate lies in the past,
    /// the usage is decreasing or not changing.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub estimated_full_date: Option<i64>,
    /// An error description, for example, when the datastore could not be looked up
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

pub const ADMIN_DATASTORE_LIST_SNAPSHOTS_RETURN_TYPE: ReturnType = ReturnType {
    optional: false,
    schema: &ArraySchema::new(
        "Returns the list of snapshots.",
        &SnapshotListItem::API_SCHEMA,
    )
    .schema(),
};

pub const ADMIN_DATASTORE_LIST_SNAPSHOT_FILES_RETURN_TYPE: ReturnType = ReturnType {
    optional: false,
    schema: &ArraySchema::new(
        "Returns the list of archive files inside a backup snapshots.",
        &BackupContent::API_SCHEMA,
    )
    .schema(),
};

pub const ADMIN_DATASTORE_LIST_GROUPS_RETURN_TYPE: ReturnType = ReturnType {
    optional: false,
    schema: &ArraySchema::new(
        "Returns the list of backup groups.",
        &GroupListItem::API_SCHEMA,
    )
    .schema(),
};

pub const ADMIN_DATASTORE_PRUNE_RETURN_TYPE: ReturnType = ReturnType {
    optional: false,
    schema: &ArraySchema::new(
        "Returns the list of snapshots and a flag indicating if there are kept or removed.",
        &PruneListItem::API_SCHEMA,
    )
    .schema(),
};
