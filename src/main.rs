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
    
    // Set up database connection pool
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
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