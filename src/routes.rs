use crate::models::{Flight, FlightQuery, InputFlight, NewFlight};
use crate::schema::flights::dsl::*;
use crate::DbPool;
use serde::Serialize;
use diesel::prelude::*;
use rocket::http::Status;
use rocket::serde::json::{json, Json, Value};
use rocket::{get, post, State};
use rocket::outcome::Outcome;
use rocket::request::{self, FromRequest, Request};

// Custom guard for API password authentication
#[allow(dead_code)]
struct ApiKey(String);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for ApiKey {
    type Error = ();

    async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        // Get password from environment variable
        let expected_password = match std::env::var("FLIGHT_API_PASSWORD") {
            Ok(pwd) => pwd,
            Err(_) => return Outcome::Error((Status::InternalServerError, ()))
        };

        // Extract password from query parameters
        let password = req.query_value::<String>("password");
        
        match password {
            Some(Ok(pwd)) if pwd == expected_password => Outcome::Success(ApiKey(pwd)),
            _ => Outcome::Error((Status::Unauthorized, ()))
        }
    }
}

#[post("/flights", format = "json", data = "<flight_data>")]
pub async fn insert_flights(
    pool: &State<DbPool>,
    flight_data: Json<Vec<InputFlight>>,

) -> Result<Json<Value>, Status> {
    // Clone the pool and convert flight data for use in the blocking task
    let pool_clone = pool.inner().clone();
    let flight_data_inner = flight_data.into_inner();
    
    match tokio::task::spawn_blocking(move || -> Result<usize, Status> {
        let mut conn = pool_clone.get().map_err(|e| {
            eprintln!("Connection pool error: {:?}", e);
            Status::ServiceUnavailable
        })?;
        
        // Delete all existing flight records
        diesel::delete(flights)
            .execute(&mut conn)
            .map_err(|e| {
                eprintln!("Delete error: {:?}", e);
                Status::InternalServerError
            })?;
        
        let new_flights: Vec<NewFlight> = flight_data_inner
            .into_iter()
            .map(NewFlight::from)
            .collect();
        
        let inserted_count = diesel::insert_into(flights)
            .values(&new_flights)
            .execute(&mut conn)
            .map_err(|e| {
                eprintln!("Insert error: {:?}", e);
                Status::InternalServerError
            })?;
            
        Ok(inserted_count)
    }).await {
        Ok(Ok(inserted_count)) => {
            Ok(Json(json!({
                "status": "success",
                "message": format!("Successfully deleted all flights and inserted {} new flights", inserted_count)
            })))
        },
        Ok(Err(status)) => Err(status),
        Err(_) => Err(Status::InternalServerError)
    }
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
    // Extract query parameters to avoid ownership issues in the async block
    let page_number = query.page.unwrap_or(1);
    let items_per_page = query.limit.unwrap_or(20).min(20); // Limit max to 20
    let origin_filter = query.origin.clone();
    let destination_filter = query.destination.clone();
    let max_price_filter = query.max_price;
    let max_rain_filter = query.max_rain;
    let sort_by = query.sort_by.clone();
    let airline_filters = query.airline.clone()
        .map(|s| s.split(',').map(|name| name.trim().to_string()).collect::<Vec<String>>());

    // Clone the pool for use in the blocking task
    let pool_clone = pool.inner().clone();
    
    // Use tokio::spawn_blocking for database operations
    match tokio::task::spawn_blocking(move || -> Result<PaginatedFlightResponse, Status> {
        let mut conn = pool_clone.get().map_err(|e| {
            eprintln!("Connection pool error: {:?}", e);
            Status::ServiceUnavailable
        })?;
        
        let offset = (page_number - 1) * items_per_page;

        // Build the main query for retrieving flights
        let mut main_query = flights.into_boxed();

        // Apply filters
        if let Some(org) = &origin_filter {
            main_query = main_query.filter(origin.eq(org));
        }

        if let Some(dest) = &destination_filter {
            main_query = main_query.filter(destination.eq(dest));
        }

        if let Some(max_p) = max_price_filter {
            main_query = main_query.filter(price_inr.le(max_p));
        }

        if let Some(max_r) = max_rain_filter {
            main_query = main_query.filter(rain_probability.le(max_r));
        }

        // Apply airline filter if present
        if let Some(airlines_list) = &airline_filters {
            if !airlines_list.is_empty() {
                main_query = main_query.filter(airline.eq_any(airlines_list));
            }
        }

        // Apply sorting
        match sort_by.as_deref() {
            Some("date") => main_query = main_query.order(date.asc()),
            _ => main_query = main_query.order(price_inr.asc()), // Default sort by price
        }

        // Execute the main query with pagination
        let results = main_query
            .limit(items_per_page)
            .offset(offset)
            .load::<Flight>(&mut conn)
            .map_err(|e| {
                eprintln!("Database error loading flights: {:?}", e);
                Status::InternalServerError
            })?;

        // Build a separate count query
        let mut count_query = flights.into_boxed();

        // Apply the same filters to the count query
        if let Some(org) = &origin_filter {
            count_query = count_query.filter(origin.eq(org));
        }

        if let Some(dest) = &destination_filter {
            count_query = count_query.filter(destination.eq(dest));
        }

        if let Some(max_p) = max_price_filter {
            count_query = count_query.filter(price_inr.le(max_p));
        }

        if let Some(max_r) = max_rain_filter {
            count_query = count_query.filter(rain_probability.le(max_r));
        }

        // Apply airline filter to the count query as well
        if let Some(airlines_list) = &airline_filters {
            if !airlines_list.is_empty() {
                count_query = count_query.filter(airline.eq_any(airlines_list));
            }
        }

        // Get the total count of matching flights
        let total_count: i64 = count_query
            .count()
            .get_result(&mut conn)
            .map_err(|e| {
                eprintln!("Database error counting flights: {:?}", e);
                Status::InternalServerError
            })?;

        // Calculate total pages
        let total_pages = (total_count as f64 / items_per_page as f64).ceil() as i64;

        // Create the paginated response
        let response = PaginatedFlightResponse {
            data: results,
            page: page_number,
            total_pages,
            total_items: total_count,
        };

        Ok(response)
    }).await {
        Ok(Ok(response)) => Ok(Json(response)),
        Ok(Err(status)) => Err(status),
        Err(_) => Err(Status::InternalServerError)
    }
}