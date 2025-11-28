// src/main.rs
mod models;
mod routes;
mod schema;
mod s3_client;

use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use lambda_http::{run, service_fn, Body, Error, Request, RequestExt, Response, Method};
use std::env;
use s3_client::S3Config;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");
pub type DbPool = r2d2::Pool<ConnectionManager<SqliteConnection>>;

const DB_PATH: &str = "/tmp/flights.db";
const DB_KEY: &str = "flights.db";

// Initialize the Database (Run migrations)
fn init_db(pool: &DbPool) {
    let mut conn = pool.get().expect("Failed to get DB connection");
    conn.run_pending_migrations(MIGRATIONS).expect("Migrations failed");
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // 1. Initialize AWS & Config
    let config = aws_config::load_from_env().await;
    let s3_client = aws_sdk_s3::Client::new(&config);
    let bucket_name = env::var("S3_BUCKET").expect("S3_BUCKET not set");
    
    let s3_cfg = S3Config {
        bucket: bucket_name,
        key: DB_KEY.to_string(),
        local_path: DB_PATH.to_string(),
    };

    // 2. Sync DB from S3 (Cold Start)
    if let Err(e) = s3_client::ensure_db_exists(&s3_client, &s3_cfg).await {
        println!("Warning: DB download failed (might be first run): {}", e);
    }

    // 3. Create Connection Pool
    let manager = ConnectionManager::<SqliteConnection>::new(DB_PATH);
    let pool = r2d2::Pool::builder()
        .max_size(1) // Keep connections low
        .build(manager)
        .expect("Failed to build pool");

    // 4. Run Migrations
    init_db(&pool);

    // 5. Start Lambda Runtime
    // We clone our state to move it into the request handler closure
    let pool_ref = pool.clone();
    let s3_client_ref = s3_client.clone();
    let s3_cfg_ref = s3_cfg.clone();

    run(service_fn(move |req: Request| {
        let pool = pool_ref.clone();
        let s3 = s3_client_ref.clone();
        let cfg = s3_cfg_ref.clone();
        
        async move {
            match (req.method(), req.uri().path()) {
                (&Method::GET, "/api/flights") => routes::get_flights(&pool, req).await,
                (&Method::POST, "/api/flights") => routes::insert_flights(&pool, &s3, &cfg, req).await,
                _ => Ok(Response::builder()
                    .status(404)
                    .body(Body::from("Not Found"))
                    .unwrap()),
            }
        }
    })).await
}