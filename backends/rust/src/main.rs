mod errors;
mod routes;
mod structs;
mod utils;

use {
    actix_cors::Cors,
    actix_web::{middleware, web, App, HttpServer},
    dotenv,
    routes::{article, auth},
    std::sync::Arc,
    structs::app_data::AppData,
};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    let data = web::Data::new(Arc::new(AppData::new()));
    HttpServer::new(move || {
        let cors = Cors::permissive();
        App::new()
            .wrap(middleware::Logger::default())
            .wrap(cors)
            .data(web::JsonConfig::default().limit(4096))
            .app_data(data.clone())
            .service(auth::handle_get_nonce)
            .service(auth::handle_login)
            .service(article::handle_get_article)
    })
    .bind("localhost:3001")?
    .run()
    .await
}
