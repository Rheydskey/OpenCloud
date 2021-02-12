use crate::lib::db::conn::conn;
use logger::error;
use sqlx::Executor;

pub async fn create() {
    let mut conn = conn().await;
    match conn
        .execute(
            "CREATE TABLE IF NOT EXISTS Log (
                  id              INTEGER PRIMARY KEY,
                  type            TEXT NOT NULL,
                  user_id         INTEGER NOT NULL,
                  date           TEXT
                  )",
        )
        .await
    {
        Ok(_) => {}
        Err(_) => error("Error on create the log database"),
    };
}
