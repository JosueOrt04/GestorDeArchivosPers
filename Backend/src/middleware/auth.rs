use actix_web::{dev::Payload, web, FromRequest, HttpRequest};
use futures::future::{ready, Ready};

use crate::{config::AppConfig, errors::ApiError, utils::jwt};

#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: String,
    pub email: String,
    pub role: String,
}

impl FromRequest for AuthUser {
    type Error = ApiError;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let cfg = req
            .app_data::<web::Data<AppConfig>>()
            .map(|d| d.get_ref().clone());

        let Some(cfg) = cfg else {
            return ready(Err(ApiError::Internal));
        };

        let auth = req
            .headers()
            .get("Authorization")
            .and_then(|h| h.to_str().ok())
            .unwrap_or("");

        if !auth.starts_with("Bearer ") {
            return ready(Err(ApiError::Unauthorized("Missing Bearer token".into())));
        }

        let token = auth.trim_start_matches("Bearer ").trim();
        match jwt::verify_jwt(token, &cfg.jwt_secret) {
            Ok(claims) => ready(Ok(AuthUser {
                user_id: claims.sub,
                email: claims.email,
                role: claims.role,
            })),
            Err(_) => ready(Err(ApiError::Unauthorized("Invalid token".into()))),
        }
    }
}
