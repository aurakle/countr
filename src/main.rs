use actix_web::{App, HttpRequest, HttpResponse, HttpServer, Responder, get, post, web::Path};
use chrono::DateTime;
use chrono::Local;
use chrono::TimeZone;
use chrono::Utc;
use config::get_config;
use db::get_pool;
use env_logger::Target;
use eyre::Context as _;
use eyre::Result;
use log::warn;
use log::{LevelFilter, error, info};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

mod config;
mod db;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub(crate) struct Entry {
    pub id: String,
    pub count: i64,
    pub modified_at: DateTime<Utc>,
}

#[actix_web::main]
async fn main() {
    env_logger::Builder::from_default_env()
        .target(Target::Stdout)
        .filter_level(LevelFilter::Info)
        .init();
    if let Err(err) = setup().await {
        error!("Error initializing: {err}");
    }
}

async fn setup() -> Result<()> {
    db::init_db().await?;

    let server = HttpServer::new(move || App::new().service(fetch).service(update))
        .bind(("0.0.0.0", get_config().port))
        .context(format!(
            "Failed to bind to address, port {} might be in use",
            get_config().port
        ))?;

    info!("Server configured, running...");
    server.run().await?;

    Ok(())
}

#[get("/{id}")]
async fn fetch(_req: HttpRequest, id: Path<String>) -> impl Responder {
    let id = id.into_inner();

    if id.len() > 64 {
        return HttpResponse::BadRequest().finish();
    }

    let entry = match db::get(get_pool().await, id).await {
        Ok(val) => val,
        Err(_) => return HttpResponse::NotFound().finish(),
    };

    HttpResponse::Ok().json(entry)
}

#[post("/{id}")]
async fn update(_req: HttpRequest, id: Path<String>) -> impl Responder {
    let id = id.into_inner();

    if id.len() > 64 {
        return HttpResponse::BadRequest().finish();
    }

    let entry = match db::update(get_pool().await, id).await {
        Ok(val) => val,
        Err(e) => {
            warn!("{:?}", e);
            return HttpResponse::InternalServerError().finish();
        },
    };

    HttpResponse::Ok().json(entry)
}
