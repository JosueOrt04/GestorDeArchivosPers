use bson::oid::ObjectId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileDoc {
    #[serde(rename = "_id")]
    pub id: ObjectId,

    pub owner_id: String, // AuthUser.user_id (hex)
    pub original_name: String,
    pub stored_name: String,
    pub mime: String,
    pub size: i64,
    pub visibility: String, // "private" | "public"
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct FileOut {
    pub id: String,
    pub owner_id: String,
    pub original_name: String,
    pub mime: String,
    pub size: i64,
    pub visibility: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<FileDoc> for FileOut {
    fn from(f: FileDoc) -> Self {
        Self {
            id: f.id.to_hex(),
            owner_id: f.owner_id,
            original_name: f.original_name,
            mime: f.mime,
            size: f.size,
            visibility: f.visibility,
            created_at: f.created_at,
            updated_at: f.updated_at,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct UpdateVisibilityDto {
    pub visibility: String, // "private" | "public"
}
