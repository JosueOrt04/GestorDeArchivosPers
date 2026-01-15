use bcrypt::{hash, verify, DEFAULT_COST};

pub fn hash_password(password: &str) -> Result<String, String> {
    hash(password, DEFAULT_COST).map_err(|e| e.to_string())
}

pub fn verify_password(password: &str, hash_str: &str) -> Result<bool, String> {
    verify(password, hash_str).map_err(|e| e.to_string())
}
