// models.rs
use crate::schema::flights;
use diesel::prelude::*;
use rocket::serde::{Deserialize, Serialize};
use rocket::form::FromForm;
use uuid::Uuid;

#[derive(Queryable, Serialize)]
pub struct Flight {
    pub id: i32,
    pub uuid: String,
    pub date: String,
    pub origin: String,
    pub destination: String,
    pub airline: String,
    pub duration: String,
    pub flight_type: String,
    pub price_inr: i32,
    pub origin_country: String,
    pub destination_country: String,
    pub link: String,
    pub rain_probability: f32,
    pub free_meal: bool,
    pub min_checked_luggage_price: Option<i32>,
    pub min_checked_luggage_weight: Option<String>,
    pub total_with_min_luggage: Option<i32>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")] // Use this to handle all camelCase keys automatically
pub struct InputFlight {
    pub uuid: Option<String>,
    pub date: String,
    pub origin: String,
    pub destination: String,
    pub airline: String,
    pub duration: String,
    pub flight_type: String,
    #[serde(rename = "price_inr")]
    pub price_inr: i32,
    pub origin_country: String,
    pub destination_country: String,
    pub link: String,
    pub rain_probability: f32,
    pub free_meal: bool,
    // Optional fields
    pub min_checked_luggage_price: Option<i32>,
    pub min_checked_luggage_weight: Option<String>,
    pub total_with_min_luggage: Option<i32>,
}

#[derive(Insertable)]
#[diesel(table_name = flights)]
pub struct NewFlight {
    pub uuid: String,
    pub date: String,
    pub origin: String,
    pub destination: String,
    pub airline: String,
    pub duration: String,
    pub flight_type: String,
    pub price_inr: i32,
    pub origin_country: String,
    pub destination_country: String,
    pub link: String,
    pub rain_probability: f32,
    pub free_meal: bool,
    pub min_checked_luggage_price: Option<i32>,
    pub min_checked_luggage_weight: Option<String>,
    pub total_with_min_luggage: Option<i32>,
}

impl From<InputFlight> for NewFlight {
    fn from(flight: InputFlight) -> Self {
        NewFlight {
            uuid: flight.uuid.unwrap_or_else(|| Uuid::new_v4().to_string()),
            date: flight.date,
            origin: flight.origin,
            destination: flight.destination,
            airline: flight.airline,
            duration: flight.duration,
            flight_type: flight.flight_type,
            price_inr: flight.price_inr,
            origin_country: flight.origin_country,
            destination_country: flight.destination_country,
            link: flight.link,
            rain_probability: flight.rain_probability,
            free_meal: flight.free_meal,
            min_checked_luggage_price: flight.min_checked_luggage_price,
            min_checked_luggage_weight: flight.min_checked_luggage_weight,
            total_with_min_luggage: flight.total_with_min_luggage,
        }
    }
}

#[derive(Deserialize, FromForm)]
pub struct FlightQuery {
    pub page: Option<i64>,
    pub limit: Option<i64>,
    pub origin: Option<String>,
    pub destination: Option<String>,
    pub sort_by: Option<String>,
    pub max_price: Option<i32>,
    pub airline: Option<String>,
    pub max_rain: Option<f32>, // Re-added query parameter
}