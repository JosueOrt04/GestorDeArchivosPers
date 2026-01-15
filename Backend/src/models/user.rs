use bson::oid::ObjectId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    #[serde(rename = "_id")]
    pub id: ObjectId,

    pub name: String,
    pub email: String,
    pub role: String, // "cliente" | "colaborador"

    // ✅ IMPORTANTE: NO uses skip_serializing aquí, si no Mongo NO lo guarda.
    pub password_hash: Option<String>,

    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct RegisterDto {
    #[validate(length(min = 2, message = "name too short"))]
    pub name: String,

    #[validate(email(message = "invalid email"))]
    pub email: String,

    #[validate(length(min = 6, message = "password too short"))]
    pub password: String,

    #[validate(length(min = 3, message = "role required"))]
    pub role: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct LoginDto {
    #[validate(email(message = "invalid email"))]
    pub email: String,

    #[validate(length(min = 6, message = "password too short"))]
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: PublicUser,
}

#[derive(Debug, Serialize)]
pub struct PublicUser {
    pub id: String,
    pub name: String,
    pub email: String,
    pub role: String,
    pub created_at: DateTime<Utc>,
}

impl From<User> for PublicUser {
    fn from(u: User) -> Self {
        Self {
            id: u.id.to_hex(),
            name: u.name,
            email: u.email,
            role: u.role,
            created_at: u.created_at,
        }
    }
}
