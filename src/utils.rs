use std::time::{SystemTime, UNIX_EPOCH};

use crypto::{digest::Digest, sha2::Sha256};

pub fn generate_challenge() -> String {
    let now_nano = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("something went wrong when calculate timestamp")
        .as_nanos();
    let mut hash_sha256 = Sha256::new();
    hash_sha256.input_str(&now_nano.to_string());
    hash_sha256.result_str()
}
