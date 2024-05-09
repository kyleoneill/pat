use rand::{distributions::Alphanumeric, Rng};
use sha2::{Sha256, Digest};

const SALT_LENGTH: usize = 12;

pub fn hash_password(mut password: String, salt: &str) -> String {
    password.push_str(salt);
    let mut hasher = Sha256::new();
    hasher.update(password);
    let result = hasher.finalize();
    format!("{:X}", result)
}

pub fn generate_salt() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(SALT_LENGTH)
        .map(char::from)
        .collect()
}
