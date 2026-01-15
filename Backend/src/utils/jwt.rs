use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // user id (ObjectId hex)
    pub email: String,
    pub role: String,
    pub exp: usize,
}

pub fn sign_jwt(user_id: &str, email: &str, role: &str, secret: &str, exp_minutes: i64) -> Result<String, String> {
    let exp = (Utc::now() + Duration::minutes(exp_minutes)).timestamp() as usize;

    let claims = Claims {
        sub: user_id.to_string(),
        email: email.to_string(),
        role: role.to_string(),
        exp,
    };

    encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_bytes()))
        .map_err(|e| e.to_string())
}

pub fn verify_jwt(token: &str, secret: &str) -> Result<Claims, String> {
    let data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|e| e.to_string())?;

    Ok(data.claims)
}
