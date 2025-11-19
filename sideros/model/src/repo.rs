use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum SyncType {
    Rsync,
    Https,
    Git,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    name: String
    file_path: Option<String>,
    location: String,
    sync_type: SyncType,
    port: Option<u16>,
    sync_uri: String,
    auto_sync: bool,
    rsync_verify_jobs: u16,
    rsync_verify_metamanifest: bool,
    rsync_verify_max_age: u16,
    openpgp_key_path: String, // = /usr/share/openpgp_keys/gentoo_release.asc
    openpgp_keyserver: String, // = hkps://keys.gentoo.org
    openpgp_key_refresh_retry_count: u8, //  = 40
    openpgp_key_refresh_retry_overall_timeout_sec: u16, // = 1200
}
// openpgp_key_refresh_retry_delay_exp_base = 2
// openpgp_key_refresh_retry_delay_max = 60
// openpgp_key_refresh_retry_delay_mult = 4
