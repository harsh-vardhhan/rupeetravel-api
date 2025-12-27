# Flights API for RupeeTravel 
![Deploy to AWS](https://github.com/harsh-vardhhan/rupeetravel-api/actions/workflows/deploy.yml/badge.svg)

## Architecture
<img width="1429" height="834" alt="architecture" src="https://github.com/user-attachments/assets/fd264671-9975-42e9-8e4f-3aabf9ea51da" />

### The "SQLite-over-S3" Pattern

This project implements a **Serverless, Shared-Nothing** architecture designed specifically for high-read, low-write workloads with relatively small datasets (e.g., < 100MB).

**How it works:**
1.  **Compute (AWS Lambda + Rust)**: The API runs on a Lambda function using the custom Rust runtime (`provided.al2023`) for minimal cold starts and memory usage.
2.  **Storage (S3 + SQLite)**: There is no central database server (like RDS). The entire database is a single `flights.db` file stored in S3.
3.  **Execution Flow**:
    -   When a Lambda instance spins up (Cold Start), it downloads the DB file from S3 to its local `/tmp` storage.
    -   It connects to this local SQLite file.
    -   Subsequent requests to the same Lambda instance query the local file directly, with zero network latency.

### Rationale

This design is an aggressive optimization for **"Read-Only"** applications that need to handle **sudden virality**.

1.  **Infinite Read Scaling (Handling Virality)**
    -   *Traditional DBs*: A viral spike spins up thousands of Lambdas, which all race to open connections to a central Postgres/MySQL server, exhausting the connection pool and crashing the app.
    -   *This Architecture*: Every Lambda has its **own private copy** of the database. If 10,000 users visit simultaneously, 100+ Lambdas spin up, each serving requests independently. There is no single bottleneck to choke the system.

2.  **Performance (Zero-Latency Queries)**
    -   Since the database lives in the Lambda's temporary storage (`/tmp`), SQL queries happen in-memory or on local disk. There is no network round-trip for data retrieval once the Lambda is warm.

3.  **Cost Efficiency**
    -   **No Idle Costs**: Unlike RDS, which charges per hour even when no one is using it, S3 storage costs fractions of a cent, and Lambda charges only when code is running.
    -   **Free Tier Friendly**: This architecture serves thousands of requests for pennies.

### Trade-offs
-   **Write Latency**: Updates are slow because the application must update the local file and upload it back to S3. This is fine for "daily update" data (like flight schedules) but bad for high-frequency writes (like chat apps).
-   **Eventual Consistency**: There is a brief window during updates where different Lambda instances might serve slightly different versions of the data.

## API Documentation

### Base URL
`https://emz8lgvgm2.execute-api.ap-south-1.amazonaws.com/api`

### Endpoints

#### 1. Get Flights
Retrieve a paginated list of flights with optional filtering and sorting.

**[Live Demo](https://emz8lgvgm2.execute-api.ap-south-1.amazonaws.com/api/flights)**


- **URL**: `/flights`
- **Method**: `GET`
- **Query Parameters**:
  - `page` (optional): Page number (default: 1)
  - `limit` (optional): Items per page (default: 20, max: 20)
  - `origin` (optional): Filter by origin city
  - `destination` (optional): Filter by destination city
  - `date` (optional): Filter by date (YYYY-MM-DD)
  - `sort_by` (optional): Sort order ('date' for date ascending, otherwise defaults to price ascending)
  - `max_price` (optional): Filter flights below a certain price
  - `airline` (optional): Filter by airline (comma-separated for multiple)
  - `max_rain` (optional): max rain probability

- **Response**:
  ```json
  {
    "data": [
      {
        "id": 1,
        "uuid": "123-uuid",
        "date": "2023-12-25",
        "origin": "Delhi",
        "destination": "Mumbai",
        "airline": "Indigo",
        "duration": "2h 10m",
        "flight_type": "Nonstop",
        "price_inr": 4500,
        "origin_country": "India",
        "destination_country": "India",
        "link": "https://google.com/flights/...",
        "rain_probability": 0.0,
        "free_meal": false,
        "min_checked_luggage_price": null,
        "min_checked_luggage_weight": null,
        "total_with_min_luggage": null
      }
    ],
    "page": 1,
    "total_pages": 5,
    "total_items": 100
  }
  ```

#### 2. Insert Flights (Admin Only)
Replace the entire flight database with new data and sync to S3.

- **URL**: `/flights`
- **Method**: `POST`
- **Authentication**: Requires `password` query parameter.
- **Body**: JSON Array of flight objects.
  ```json
  [
    {
      "date": "2023-12-25",
      "origin": "Delhi",
      "destination": "Mumbai",
      "airline": "Indigo",
      "duration": "2h 10m",
      "flight_type": "Nonstop",
      "price_inr": 4500,
      "origin_country": "India",
      "destination_country": "India",
      "link": "https://google.com/flights/...",
      "rain_probability": 0.0,
      "free_meal": false
    }
  ]
  ```

- **Response**:
  ```json
  {
    "status": "success",
    "inserted": 50
  }
  ```
