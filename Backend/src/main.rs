mod config;
mod db;
mod errors;
mod middleware;
mod models;
mod routes;
mod utils;

use actix_cors::Cors;
use actix_web::{http::header, middleware::Logger, web, App, HttpResponse, HttpServer};
use config::AppConfig;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();
    env_logger::init();

    let cfg = AppConfig::from_env();
    let host = cfg.host.clone();
    let port = cfg.port;

    let mongo = db::mongo_client(&cfg.mongodb_uri).await;
    let state = db::AppState::new(mongo, &cfg.mongodb_db);

    println!("PCOSEW Backend running at http://{}:{}", host, port);

    let cfg_data = cfg.clone();

    HttpServer::new(move || {
        // ✅ CORS definitivo para DEV:
        // Permite localhost:5173 y 127.0.0.1:5173 sin depender de cómo abras el frontend
        let cors = Cors::default()
            .allowed_origin_fn(|origin, _req_head| {
                origin.as_bytes() == b"http://127.0.0.1:5173"
                    || origin.as_bytes() == b"http://localhost:5173"
            })
            .allowed_methods(vec!["GET", "POST", "PATCH", "DELETE", "OPTIONS"])
            .allowed_headers(vec!["Authorization", "Content-Type"])
            .allowed_header(header::ACCEPT)
            .allowed_header(header::ORIGIN)
            .max_age(3600);

        App::new()
            .wrap(Logger::default())
            .wrap(cors)
            .app_data(web::Data::new(cfg_data.clone()))
            .app_data(web::Data::new(state.clone()))
            // Health check para navegador
            .route(
                "/",
                web::get().to(|| async {
                    HttpResponse::Ok().json(serde_json::json!({
                        "name": "PCOSEW API",
                        "status": "ok",
                        "endpoints": {
                            "register": "POST /api/auth/register",
                            "login": "POST /api/auth/login",
                            "me": "POST /api/auth/me",
                            "files_list": "GET /api/files",
                            "files_upload": "POST /api/files/upload"
                        }
                    }))
                }),
            )
            .service(web::scope("/api").configure(routes::configure))
    })
    .bind((host.as_str(), port))?
    .run()
    .await
}
