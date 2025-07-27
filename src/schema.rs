// schema.rs

use diesel::prelude::*;

table! {
    flights (id) {
        id -> Integer,
        uuid -> Text,
        date -> Text,
        origin -> Text,
        destination -> Text,
        airline -> Text,
        duration -> Text,
        flight_type -> Text,
        price_inr -> Integer,
        origin_country -> Text,
        destination_country -> Text,
        link -> Text, // New field
        rain_probability -> Float, // Field is back
        free_meal -> Bool, // Field is back
        min_checked_luggage_price -> Nullable<Integer>, // New optional field
        min_checked_luggage_weight -> Nullable<Text>, // New optional field
        total_with_min_luggage -> Nullable<Integer>, // New optional field
    }
}