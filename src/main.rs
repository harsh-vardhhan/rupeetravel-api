#[macro_use]
extern crate rocket;
#[macro_use]
extern crate diesel;
extern crate dotenv;

mod models;
mod routes;
mod schema;

use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use dotenv::dotenv;
use routes::{get_flights, insert_flights};
use std::env;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

#[launch]
fn rocket() -> _ {
    dotenv().ok();
    
    // Auto-detect Railway environment
    // Railway sets RAILWAY_ENVIRONMENT and RAILWAY_PROJECT_ID variables
    let is_on_railway = env::var("RAILWAY_ENVIRONMENT").is_ok() || env::var("RAILWAY_PROJECT_ID").is_ok();
    
    // Determine the database URL based on environment
    let database_url = if is_on_railway {
        // We're on Railway, so use Railway SQLite database
        println!("Running on Railway - using Railway SQLite database");
        String::from("sqlite://sqlite3.railway.internal/database.db")
    } else {
        // Not on Railway, so use local database
        println!("Running locally - using local SQLite database");
        env::var("DATABASE_URL").expect("DATABASE_URL must be set")
    };
    
    println!("Connecting to database: {}", database_url);
    
    // Set up database connection pool
    let manager = ConnectionManager::<SqliteConnection>::new(database_url);
    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool");
    
    // Run migrations
    let mut conn = pool.get().expect("Failed to get DB connection for migrations");
    conn.run_pending_migrations(MIGRATIONS).expect("Failed to run migrations");
    
    rocket::build()
        .manage(pool)
        .mount("/api", routes![insert_flights, get_flights])
}