use actix_web::{
    middleware::{Compress, Logger},
    web::{scope, Data},
    App, HttpServer,
};
use env_logger::{init_from_env, Env};
use retreive::{config::Config, db::DbPool, routes::v1_routes};
use std::io::{Error, ErrorKind, Result};

#[actix_web::main]
async fn main() -> Result<()> {
    init_from_env(Env::new().default_filter_or("info"));

    let config = Config::build().map_err(|e| Error::new(ErrorKind::Other, e))?;

    let pool = DbPool::new(config.database_url)
        .await
        .map_err(|e| Error::new(ErrorKind::Other, e))?;

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .wrap(Compress::default())
            .app_data(Data::new(pool.clone()))
            .service(scope("/v1").configure(v1_routes))
    })
    .bind(format!("0.0.0.0:{}", config.server_port))?
    .workers(config.workers)
    .run()
    .await
}
