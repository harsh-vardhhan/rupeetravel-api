use crate::models::{Flight, FlightQuery, InputFlight, NewFlight};
use crate::schema::flights::dsl::*;
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use rocket::http::Status;
use rocket::serde::json::{json, Json, Value};
use rocket::{get, post, State};
use std::ops::Bound::Included;

type DbPool = r2d2::Pool<ConnectionManager<SqliteConnection>>;

// POST endpoint to insert flight data
#[post("/flights", format = "json", data = "<flight_data>")]
pub async fn insert_flights(
    pool: &State<DbPool>,
    flight_data: Json<Vec<InputFlight>>,
) -> Result<Json<Value>, Status> {
    let mut conn = pool.get().map_err(|_| Status::ServiceUnavailable)?;
    
    let new_flights: Vec<NewFlight> = flight_data
        .into_inner()
        .into_iter()
        .map(NewFlight::from)
        .collect();
    
    let inserted_count = diesel::insert_into(flights)
        .values(&new_flights)
        .execute(&mut conn)
        .map_err(|_| Status::InternalServerError)?;
    
    Ok(Json(json!({
        "status": "success",
        "message": format!("Successfully inserted {} flights", inserted_count)
    })))
}

// GET endpoint to fetch and filter flight data
#[get("/flights?<query..>")]
pub async fn get_flights(
    pool: &State<DbPool>,
    query: FlightQuery,
) -> Result<Json<Vec<Flight>>, Status> {
    let mut conn = pool.get().map_err(|_| Status::ServiceUnavailable)?;
    
    // Set defaults for pagination
    let page_number = query.page.unwrap_or(1);
    let items_per_page = query.limit.unwrap_or(20).min(20); // Limit max to 20
    let offset = (page_number - 1) * items_per_page;
    
    // Start building the query
    let mut query_builder = flights.into_boxed();
    
    // Apply origin and destination filters if provided
    if let Some(org) = query.origin {
        query_builder = query_builder.filter(origin.eq(org));
    }
    
    if let Some(dest) = query.destination {
        query_builder = query_builder.filter(destination.eq(dest));
    }
    
    // Apply price filter if provided
    if let Some(max_p) = query.max_price {
        query_builder = query_builder.filter(price_inr.le(max_p));
    }
    
    // Apply rain probability filter if provided
    if let Some(max_r) = query.max_rain {
        query_builder = query_builder.filter(rain_probability.le(max_r));
    }
    
    // Apply sorting
    match query.sort_by.as_deref() {
        Some("date") => query_builder = query_builder.order(date.asc()),
        _ => query_builder = query_builder.order(price_inr.asc()), // Default sort by price
    }
    
    // Execute the query with pagination
    let results = query_builder
        .limit(items_per_page)
        .offset(offset)
        .load::<Flight>(&mut conn)
        .map_err(|_| Status::InternalServerError)?;
    
    Ok(Json(results))
}