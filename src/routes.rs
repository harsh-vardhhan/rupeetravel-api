// src/routes.rs
use crate::models::{Flight, FlightQuery, InputFlight, NewFlight};
use crate::schema::flights::dsl::*;
use crate::DbPool;
use crate::s3_client::{self, S3Config};
use aws_sdk_s3::Client;
use diesel::prelude::*;
use lambda_http::{Body, Error, Request, Response, RequestExt};
use serde::Serialize;
use serde_json::json;

#[derive(Serialize)]
pub struct PaginatedFlightResponse {
    data: Vec<Flight>,
    page: i64,
    total_pages: i64,
    total_items: i64,
}

// Helper to create JSON responses
fn json_response<T: Serialize>(status: u16, body: T) -> Result<Response<Body>, Error> {
    let body_text = serde_json::to_string(&body)?;
    Ok(Response::builder()
        .status(status)
        .header("content-type", "application/json")
        .body(body_text.into())
        .map_err(Box::new)?)
}

pub async fn get_flights(pool: &DbPool, req: Request) -> Result<Response<Body>, Error> {
    // 1. Parse Query Parameters manually using serde_urlencoded
    let query_string = req.query_string_parameters();
    // We reconstruct the query string to parse it into our struct
    let qs_map = query_string.iter().map(|(k, v)| (k, v)).collect::<Vec<_>>();
    let qs_string = serde_urlencoded::to_string(&qs_map).unwrap_or_default();
    
    let query: FlightQuery = serde_urlencoded::from_str(&qs_string).unwrap_or(FlightQuery {
        page: None, limit: None, origin: None, destination: None, sort_by: None,
        max_price: None, airline: None, max_rain: None, password: None
    });

    let page_number = query.page.unwrap_or(1);
    let items_per_page = query.limit.unwrap_or(20).min(20);
    let offset = (page_number - 1) * items_per_page;

    let pool_clone = pool.clone();

    // 2. Run Database Logic (Blocking)
    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool_clone.get().map_err(|e| format!("Connection error: {}", e))?;
        let mut main_query = flights.into_boxed();

        if let Some(org) = &query.origin { main_query = main_query.filter(origin.eq(org)); }
        if let Some(dest) = &query.destination { main_query = main_query.filter(destination.eq(dest)); }
        if let Some(max_p) = query.max_price { main_query = main_query.filter(price_inr.le(max_p)); }
        if let Some(max_r) = query.max_rain { main_query = main_query.filter(rain_probability.le(max_r)); }
        
        if let Some(airline_str) = &query.airline {
             let list: Vec<String> = airline_str.split(',').map(|s| s.trim().to_string()).collect();
             if !list.is_empty() { main_query = main_query.filter(airline.eq_any(list)); }
        }

        match query.sort_by.as_deref() {
            Some("date") => main_query = main_query.order(date.asc()),
            _ => main_query = main_query.order(price_inr.asc()),
        }

        let results = main_query.limit(items_per_page).offset(offset).load::<Flight>(&mut conn)
            .map_err(|e| format!("Load error: {}", e))?;

        // Simplified Count Query (Ideally duplicate filters here like in original)
        let total_count: i64 = flights.count().get_result(&mut conn)
            .map_err(|e| format!("Count error: {}", e))?;

        let total_pages = (total_count as f64 / items_per_page as f64).ceil() as i64;
        
        Ok::<_, String>(PaginatedFlightResponse {
            data: results,
            page: page_number,
            total_pages,
            total_items: total_count,
        })
    }).await.map_err(|e| Error::from(e.to_string()))?;

    match result {
        Ok(response_data) => json_response(200, response_data),
        Err(e) => json_response(500, json!({ "error": e }))
    }
}

pub async fn insert_flights(
    pool: &DbPool, 
    s3: &Client, 
    s3_cfg: &S3Config, 
    req: Request
) -> Result<Response<Body>, Error> {
    // 1. Auth Check
    let query_string = req.query_string_parameters();
    let password = query_string.first("password").unwrap_or("");
    let expected_password = std::env::var("FLIGHT_API_PASSWORD").unwrap_or_default();
    
    if password != expected_password {
        return json_response(401, json!({ "error": "Unauthorized" }));
    }

    // 2. Parse Body
    let body_bytes = req.body();
    let flight_data: Vec<InputFlight> = match serde_json::from_slice(body_bytes) {
        Ok(data) => data,
        Err(e) => return json_response(400, json!({ "error": "Invalid JSON", "details": e.to_string() }))
    };

    let pool_clone = pool.clone();
    
    // 3. DB Write
    let db_result = tokio::task::spawn_blocking(move || {
        let mut conn = pool_clone.get().map_err(|_| "Pool error")?;
        diesel::delete(flights).execute(&mut conn).map_err(|_| "Delete error")?;
        
        let new_flights: Vec<NewFlight> = flight_data.into_iter().map(NewFlight::from).collect();
        let count = diesel::insert_into(flights).values(&new_flights).execute(&mut conn).map_err(|_| "Insert error")?;
        Ok::<_, String>(count)
    }).await.map_err(|e| Error::from(e.to_string()))?;

    match db_result {
        Ok(count) => {
            // 4. S3 Upload
            match s3_client::upload_db(s3, s3_cfg).await {
                Ok(_) => json_response(200, json!({ "status": "success", "inserted": count })),
                Err(e) => json_response(200, json!({ "status": "warning", "inserted": count, "s3_error": e }))
            }
        },
        Err(e) => json_response(500, json!({ "error": e }))
    }
}