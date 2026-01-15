use actix_multipart::Multipart;
use actix_web::{delete, get, patch, post, web, HttpResponse};
use bson::{doc, oid::ObjectId};
use chrono::Utc;
use futures::StreamExt;
use sanitize_filename::sanitize;
use std::{fs, io::Write, path::Path};

use crate::{
    db::AppState,
    errors::ApiError,
    middleware::auth::AuthUser,
    models::file::{FileDoc, FileOut, UpdateVisibilityDto},
};

const UPLOAD_DIR: &str = "uploads";

fn files_collection(state: &AppState) -> mongodb::Collection<FileDoc> {
    state.db.collection::<FileDoc>("files")
}

fn ensure_upload_dir() -> Result<(), ApiError> {
    if !Path::new(UPLOAD_DIR).exists() {
        fs::create_dir_all(UPLOAD_DIR).map_err(|_| ApiError::Internal)?;
    }
    Ok(())
}

#[post("/upload")]
pub async fn upload_file(
    user: AuthUser,
    state: web::Data<AppState>,
    mut payload: Multipart,
) -> Result<HttpResponse, ApiError> {
    ensure_upload_dir()?;

    let mut saved: Option<FileDoc> = None;

    while let Some(item) = payload.next().await {
        let mut field = item.map_err(|_| ApiError::BadRequest("Invalid multipart".into()))?;

        // ✅ En tu versión: content_disposition() regresa referencia, no Option
        let cd = field.content_disposition();

        let filename = cd
            .get_filename()
            .map(|f| sanitize(f))
            .unwrap_or_else(|| "file.bin".to_string());

        let stored_name = format!("{}_{}", ObjectId::new().to_hex(), filename);
        let filepath = format!("{}/{}", UPLOAD_DIR, stored_name);

        // ✅ En tu versión: content_type() es Option<&Mime>
        let mime = field
            .content_type()
            .map(|m| m.to_string())
            .unwrap_or_else(|| "application/octet-stream".to_string());

        let mut f = fs::File::create(&filepath).map_err(|_| ApiError::Internal)?;
        let mut size: i64 = 0;

        while let Some(chunk) = field.next().await {
            let data = chunk.map_err(|_| ApiError::Internal)?;
            size += data.len() as i64;
            f.write_all(&data).map_err(|_| ApiError::Internal)?;
        }

        let now = Utc::now();
        let doc = FileDoc {
            id: ObjectId::new(),
            owner_id: user.user_id.clone(),
            original_name: filename,
            stored_name,
            mime,
            size,
            visibility: "private".to_string(),
            created_at: now,
            updated_at: now,
        };

        let col = files_collection(&state);
        col.insert_one(&doc, None)
            .await
            .map_err(|e| {
                eprintln!("Mongo insert file error: {:?}", e);
                ApiError::Internal
            })?;

        saved = Some(doc);
        break; // solo 1 archivo por request
    }

    let Some(saved) = saved else {
        return Err(ApiError::BadRequest("No file uploaded".into()));
    };

    Ok(HttpResponse::Created().json(FileOut::from(saved)))
}

#[get("")]
pub async fn list_files(
    user: AuthUser,
    state: web::Data<AppState>,
) -> Result<HttpResponse, ApiError> {
    let col = files_collection(&state);

    let mut cursor = col
        .find(doc! { "owner_id": &user.user_id }, None)
        .await
        .map_err(|_| ApiError::Internal)?;

    let mut out: Vec<FileOut> = Vec::new();
    while let Some(item) = cursor.next().await {
        let f = item.map_err(|_| ApiError::Internal)?;
        out.push(FileOut::from(f));
    }

    Ok(HttpResponse::Ok().json(out))
}

#[get("/{id}/download")]
pub async fn download_file(
    user: AuthUser,
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let id = ObjectId::parse_str(path.into_inner())
        .map_err(|_| ApiError::BadRequest("Invalid file id".into()))?;

    let col = files_collection(&state);
    let file = col
        .find_one(doc! { "_id": id, "owner_id": &user.user_id }, None)
        .await
        .map_err(|_| ApiError::Internal)?
        .ok_or_else(|| ApiError::NotFound("File not found".into()))?;

    let filepath = format!("{}/{}", UPLOAD_DIR, file.stored_name);
    let bytes =
        fs::read(&filepath).map_err(|_| ApiError::NotFound("File missing on disk".into()))?;

    Ok(HttpResponse::Ok()
        .insert_header(("Content-Type", file.mime))
        .insert_header((
            "Content-Disposition",
            format!("attachment; filename=\"{}\"", file.original_name),
        ))
        .body(bytes))
}

#[patch("/{id}/visibility")]
pub async fn update_visibility(
    user: AuthUser,
    state: web::Data<AppState>,
    path: web::Path<String>,
    body: web::Json<UpdateVisibilityDto>,
) -> Result<HttpResponse, ApiError> {
    let id = ObjectId::parse_str(path.into_inner())
        .map_err(|_| ApiError::BadRequest("Invalid file id".into()))?;

    let visibility = body.visibility.trim().to_lowercase();
    if visibility != "public" && visibility != "private" {
        return Err(ApiError::BadRequest("visibility must be public|private".into()));
    }

    let col = files_collection(&state);
    let res = col
        .update_one(
            doc! { "_id": id, "owner_id": &user.user_id },
            doc! { "$set": { "visibility": &visibility, "updated_at": Utc::now() } },
            None,
        )
        .await
        .map_err(|_| ApiError::Internal)?;

    if res.matched_count == 0 {
        return Err(ApiError::NotFound("File not found".into()));
    }

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "ok": true,
        "visibility": visibility
    })))
}

#[delete("/{id}")]
pub async fn delete_file(
    user: AuthUser,
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let id = ObjectId::parse_str(path.into_inner())
        .map_err(|_| ApiError::BadRequest("Invalid file id".into()))?;

    let col = files_collection(&state);
    let file = col
        .find_one(doc! { "_id": id, "owner_id": &user.user_id }, None)
        .await
        .map_err(|_| ApiError::Internal)?
        .ok_or_else(|| ApiError::NotFound("File not found".into()))?;

    let filepath = format!("{}/{}", UPLOAD_DIR, file.stored_name);
    let _ = fs::remove_file(&filepath);

    col.delete_one(doc! { "_id": id, "owner_id": &user.user_id }, None)
        .await
        .map_err(|_| ApiError::Internal)?;

    Ok(HttpResponse::Ok().json(serde_json::json!({ "ok": true })))
}
