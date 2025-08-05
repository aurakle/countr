use actix_web::{get, post, web::Path, App, HttpRequest, HttpResponse, HttpServer, Responder};
use chrono::DateTime;
use config::get_config;
use db::get_pool;
use env_logger::Target;
use eyre::Context as _;
use log::{error, info, LevelFilter};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

mod db;
mod config;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub(crate) struct Entry {
    pub id: String,
    pub count: i64,
    pub modified_at: DateTime,
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

    let server = {
        HttpServer::new(move || {
            App::new()
                .service(fetch)
                .service(update)
        })
        .bind(("0.0.0.0", get_config().port))
        .context(format!(
            "Failed to bind to address, port {} might be in use",
            get_config().port
        ))?
    };

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

    let entry = match increment_or_create(id).await {
        Ok(val) => val,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    HttpResponse::Ok().json(entry)
}

async fn increment_or_create(id: String) -> Result<Entry> {
    let mut trans = get_pool().await.begin().await?;

    let entry = match db::get(&mut *trans, id).await {
        Ok(val) => val,
        Err(_) => {
            let entry = Entry {
                id,
                count: 0,
                modified_at: todo!(),
            };

            db::create(&mut *trans, entry.clone()).await?;

            entry
        },
    };

    let entry = Entry {
        id,
        count: entry.count + 1,
        modified_at: todo!(),
    };

    db::delete(&mut *trans, id).await?;
    db::create(&mut *trans, entry.clone()).await?;

    trans.commit().await?;

    Ok(entry)
}
