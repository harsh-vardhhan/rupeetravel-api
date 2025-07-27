-- In your up.sql file

CREATE TABLE flights (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    uuid TEXT NOT NULL UNIQUE,
    date TEXT NOT NULL,
    origin TEXT NOT NULL,
    destination TEXT NOT NULL,
    airline TEXT NOT NULL,
    duration TEXT NOT NULL,
    flight_type TEXT NOT NULL,
    price_inr INTEGER NOT NULL,
    origin_country TEXT NOT NULL,
    destination_country TEXT NOT NULL,
    link TEXT NOT NULL,
    rain_probability REAL NOT NULL,
    free_meal BOOLEAN NOT NULL,
    min_checked_luggage_price INTEGER,
    min_checked_luggage_weight TEXT,
    total_with_min_luggage INTEGER
);