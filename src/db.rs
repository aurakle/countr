use std::sync::OnceLock;

use chrono::DateTime;
use chrono::Utc;
use eyre::Context as _;
use eyre::Result;
use serde::{Deserialize, Serialize};
use sqlx::{Executor, FromRow, Pool, Postgres, postgres::PgPoolOptions};

use crate::{Entry, config::get_config};

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
        modified_at TIMESTAMPTZ NOT NULL
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

pub(crate) async fn get(
    executor: impl Executor<'_, Database = Postgres>,
    id: String,
) -> Result<Entry> {
    sqlx::query_as::<Postgres, Entry>(
        "
SELECT * FROM entries
WHERE id = $1
",
    )
    .bind(id)
    .fetch_one(executor)
    .await
    .context("Entry does not exist")
}

pub(crate) async fn update(
    executor: impl Executor<'_, Database = Postgres>,
    id: String,
) -> Result<Entry> {
    sqlx::query_as::<Postgres, Entry>(
        "
INSERT INTO entries (id, count, modified_at)
VALUES ($1, 1, $2)
ON CONFLICT (id)
DO UPDATE SET count = entries.count + 1, modified_at = $2
RETURNING *;
",
    )
    .bind(id)
    .bind(Utc::now())
    .fetch_one(executor)
    .await
    .context("Failed to update entry")
}
