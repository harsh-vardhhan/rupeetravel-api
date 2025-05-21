mod models;
mod routes;
mod schema;

use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use dotenv::dotenv;
use rocket::data::{Limits, ToByteUnit};
use rocket::fairing::AdHoc;
use rocket::{launch, routes, Build, Rocket};
use routes::{get_flights, insert_flights};
use std::env;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

pub type DbPool = r2d2::Pool<ConnectionManager<SqliteConnection>>;

async fn run_migrations(rocket: Rocket<Build>) -> Result<Rocket<Build>, Rocket<Build>> {
    let pool = rocket.state::<DbPool>().expect("Database pool not found");
    let pool_clone = pool.clone();
    let result = tokio::task::spawn_blocking(
        move || -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            let mut conn = pool_clone.get()?;
            conn.run_pending_migrations(MIGRATIONS)?;
            Ok(())
        },
    )
    .await;

    match result {
        Ok(Ok(_)) => {
            println!("Migrations executed successfully!");
            Ok(rocket)
        }
        Ok(Err(e)) => {
            eprintln!("Failed to run migrations: {:?}", e);
            Err(rocket)
        }
        Err(e) => {
            eprintln!("Task join error: {:?}", e);
            Err(rocket)
        }
    }
}

fn init_db_pool() -> DbPool {
    dotenv().ok();

    let database_url = match env::var("DATABASE_URL") {
        Ok(url) => {
            println!("Using database URL from environment: {}", url);
            url
        }
        Err(_) => {
            let is_on_railway =
                env::var("RAILWAY_ENVIRONMENT").is_ok() || env::var("RAILWAY_PROJECT_ID").is_ok();

            if is_on_railway {
                println!("Running on Railway - using Railway SQLite database");
                String::from("sqlite3.railway.internal")
            } else {
                panic!("DATABASE_URL environment variable must be set");
            }
        }
    };

    println!("Connecting to database: {}", database_url);

    let manager = ConnectionManager::<SqliteConnection>::new(database_url);
    r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool")
}

#[launch]
fn rocket() -> _ {
    let db_pool = init_db_pool();

    rocket::build()
        .manage(db_pool)
        .configure(
            rocket::Config::figment()
                .merge(("limits", Limits::new().limit("json", 15.mebibytes()))),
        )
        .attach(AdHoc::try_on_ignite("Database Migrations", run_migrations))
        .mount("/api", routes![insert_flights, get_flights])
}
