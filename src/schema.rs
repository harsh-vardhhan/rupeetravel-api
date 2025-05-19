table! {
    flights (id) {
        id -> Integer,
        uuid -> Text,
        date -> Text,
        origin -> Text,
        destination -> Text,
        airline -> Text,
        time -> Text,
        duration -> Text,
        flight_type -> Text,
        price_inr -> Integer,
        origin_country -> Text,
        destination_country -> Text,
        rain_probability -> Float,
        free_meal -> Bool,
    }
}