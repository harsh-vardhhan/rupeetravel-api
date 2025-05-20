use crate::models::{Flight, FlightQuery, InputFlight, NewFlight};
use crate::schema::flights::dsl::*;
use serde::Serialize;
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use rocket::http::Status;
use rocket::serde::json::{json, Json, Value};
use rocket::{get, post, State};

type DbPool = r2d2::Pool<ConnectionManager<SqliteConnection>>;

#[post("/flights?<password>", format = "json", data = "<flight_data>")]
pub async fn insert_flights(
    pool: &State<DbPool>,
    flight_data: Json<Vec<InputFlight>>,
    password: Option<String>,
) -> Result<Json<Value>, Status> {
    // Get password from environment variable
    let env_password = std::env::var("FLIGHT_API_PASSWORD")
        .map_err(|_| Status::InternalServerError)?;
    
    // Verify password
    match password {
        Some(pwd) if pwd == env_password => {},
        _ => return Err(Status::Unauthorized)
    }
    
    let mut conn = pool.get().map_err(|_| Status::ServiceUnavailable)?;
    
    // Delete all existing flight records
    diesel::delete(flights)
        .execute(&mut conn)
        .map_err(|_| Status::InternalServerError)?;
    
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
        "message": format!("Successfully deleted all flights and inserted {} new flights", inserted_count)
    })))
}

#[derive(Serialize)]
pub struct PaginatedFlightResponse {
    data: Vec<Flight>,
    page: i64,
    total_pages: i64,
    total_items: i64,
}

#[get("/flights?<query..>")]
pub async fn get_flights(
    pool: &State<DbPool>,
    query: FlightQuery,
) -> Result<Json<PaginatedFlightResponse>, Status> {
    let mut conn = pool.get().map_err(|_| Status::ServiceUnavailable)?;
    
    // Set defaults for pagination
    let page_number = query.page.unwrap_or(1);
    let items_per_page = query.limit.unwrap_or(20).min(20); // Limit max to 20
    let offset = (page_number - 1) * items_per_page;
    
    // Extract filter values to avoid ownership issues
    let origin_filter = query.origin.as_ref();
    let destination_filter = query.destination.as_ref();
    let max_price_filter = query.max_price;
    let max_rain_filter = query.max_rain;
    let sort_by = query.sort_by.as_deref();
    
    // Build the main query for retrieving flights
    let mut main_query = flights.into_boxed();
    
    // Apply filters
    if let Some(org) = origin_filter {
        main_query = main_query.filter(origin.eq(org));
    }
    
    if let Some(dest) = destination_filter {
        main_query = main_query.filter(destination.eq(dest));
    }
    
    if let Some(max_p) = max_price_filter {
        main_query = main_query.filter(price_inr.le(max_p));
    }
    
    if let Some(max_r) = max_rain_filter {
        main_query = main_query.filter(rain_probability.le(max_r));
    }
    
    // Apply sorting
    match sort_by {
        Some("date") => main_query = main_query.order(date.asc()),
        _ => main_query = main_query.order(price_inr.asc()), // Default sort by price
    }
    
    // Execute the main query with pagination
    let results = main_query
        .limit(items_per_page)
        .offset(offset)
        .load::<Flight>(&mut conn)
        .map_err(|_| Status::InternalServerError)?;
    
    // Build a separate count query
    let mut count_query = flights.into_boxed();
    
    // Apply the same filters to the count query
    if let Some(org) = origin_filter {
        count_query = count_query.filter(origin.eq(org));
    }
    
    if let Some(dest) = destination_filter {
        count_query = count_query.filter(destination.eq(dest));
    }
    
    if let Some(max_p) = max_price_filter {
        count_query = count_query.filter(price_inr.le(max_p));
    }
    
    if let Some(max_r) = max_rain_filter {
        count_query = count_query.filter(rain_probability.le(max_r));
    }
    
    // Get the total count of matching flights
    let total_count: i64 = count_query
        .count()
        .get_result(&mut conn)
        .map_err(|_| Status::InternalServerError)?;
    
    // Calculate total pages
    let total_pages = (total_count as f64 / items_per_page as f64).ceil() as i64;
    
    // Create the paginated response
    let response = PaginatedFlightResponse {
        data: results,
        page: page_number,
        total_pages,
        total_items: total_count,
    };
    
    Ok(Json(response))
}