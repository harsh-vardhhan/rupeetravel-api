CREATE TABLE flights (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    uuid TEXT NOT NULL UNIQUE,
    date TEXT NOT NULL,
    origin TEXT NOT NULL,
    destination TEXT NOT NULL,
    airline TEXT NOT NULL,
    time TEXT NOT NULL,
    duration TEXT NOT NULL,
    flight_type TEXT NOT NULL,
    price_inr INTEGER NOT NULL,
    origin_country TEXT NOT NULL,
    destination_country TEXT NOT NULL,
    rain_probability REAL NOT NULL,
    free_meal BOOLEAN NOT NULL DEFAULT 0
);
