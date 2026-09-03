use redis::{Client, aio::ConnectionManager};
use sqlx::{PgPool, postgres::PgPoolOptions};

use crate::utils;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub redis: ConnectionManager,
}

pub async fn setup() -> AppState {
    let db_url = utils::get_env("DATABASE_URL").expect("Missing DATABASE_URL in env");
    let pool = PgPoolOptions::new()
        .connect(&db_url)
        .await
        .expect("Failed to connect to DB");

    let redis_url = utils::get_env("REDIS_URL").expect("Missing REDIS_URL in env");
    let redis_client = Client::open(redis_url).expect("Failed to create Redis client");
    let redis = ConnectionManager::new(redis_client)
        .await
        .expect("Failed to connect to Redis");

    AppState { pool, redis }
}
