pub mod auth;
pub mod files;

use actix_web::web;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/auth").configure(auth::configure));
    cfg.service(
        web::scope("/files")
            .service(files::upload_file)
            .service(files::list_files)
            .service(files::download_file)
            .service(files::update_visibility)
            .service(files::delete_file),
    );
}
