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
    pub time: String,
    pub duration: String,
    pub flight_type: String,
    pub price_inr: i32,
    pub origin_country: String,
    pub destination_country: String,
    pub rain_probability: f32,
    pub free_meal: bool,
}

#[derive(Deserialize)]
pub struct InputFlight {
    pub uuid: Option<String>,
    pub date: String,
    pub origin: String,
    pub destination: String,
    pub airline: String,
    pub time: String,
    pub duration: String,
    #[serde(rename = "flightType")]
    pub flight_type: String,
    pub price_inr: i32,
    #[serde(rename = "originCountry")]
    pub origin_country: String,
    #[serde(rename = "destinationCountry")]
    pub destination_country: String,
    #[serde(rename = "rainProbability")]
    pub rain_probability: f32,
    #[serde(rename = "freeMeal", default)]
    pub free_meal: bool,
}

#[derive(Insertable)]
#[diesel(table_name = flights)]
pub struct NewFlight {
    pub uuid: String,
    pub date: String,
    pub origin: String,
    pub destination: String,
    pub airline: String,
    pub time: String,
    pub duration: String,
    pub flight_type: String,
    pub price_inr: i32,
    pub origin_country: String,
    pub destination_country: String,
    pub rain_probability: f32,
    pub free_meal: bool,
}

impl From<InputFlight> for NewFlight {
    fn from(flight: InputFlight) -> Self {
        NewFlight {
            uuid: flight.uuid.unwrap_or_else(|| Uuid::new_v4().to_string()),
            date: flight.date,
            origin: flight.origin,
            destination: flight.destination,
            airline: flight.airline,
            time: flight.time,
            duration: flight.duration,
            flight_type: flight.flight_type,
            price_inr: flight.price_inr,
            origin_country: flight.origin_country,
            destination_country: flight.destination_country,
            rain_probability: flight.rain_probability,
            free_meal: flight.free_meal,
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
    pub max_rain: Option<f32>,
}
