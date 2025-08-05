use std::sync::OnceLock;

use chrono::DateTime;
use eyre::Context as _;
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgPoolOptions, Executor, FromRow, Pool, Postgres};

use crate::{config::get_config, Entry};

pub(crate) type Db = Pool<Postgres>;

pub(crate) async fn get_pool() -> &'static Db {
    static POOL: OnceLock<Db> = OnceLock::new();
    POOL.get_or_init(|| {
        println!(
            "Connecting to Postgres database... (\"{}\")",
            get_config().postgres_db
        );
        PgPoolOptions::new()
            .max_connections(5)
            .connect_lazy(&get_config().postgres_db)
            .expect("Failed to connect to Postgres database")
    })
}

pub(crate) async fn init_db() -> Result<()> {
    let mut trans = get_pool().await.begin().await?;

    sqlx::query(
        "
CREATE TABLE IF NOT EXISTS entries (
        id VARCHAR(64) PRIMARY KEY,
        count BIGINT NOT NULL,
        modified_at TIMESTAMP WITH TIME ZONE NOT NULL
)
        ",
    )
        .execute(&mut *trans)
        .await?;

    trans
        .commit()
        .await
        .context("Failed to initialize database")
}

pub(crate) async fn get(executor: impl Executor<'_, Database = Postgres>, id: String) -> Result<Entry> {
    todo!()
}

pub(crate) async fn create(executor: impl Executor<'_, Database = Postgres>, entry: Entry) -> Result<()> {
    todo!()
}

pub(crate) async fn delete(executor: impl Executor<'_, Database = Postgres>, id: String) -> Result<()> {
    todo!()
}
