# Flights API for RupeeTravel 

![Deploy to AWS](https://github.com/harsh-vardhhan/rupeetravel-api/actions/workflows/deploy.yml/badge.svg)

## Architecture

<img width="1429" height="834" alt="architecture" src="https://github.com/user-attachments/assets/fd264671-9975-42e9-8e4f-3aabf9ea51da" />






## API Documentation

### Base URL
`/api`

### Endpoints

#### 1. Get Flights
Retrieve a paginated list of flights with optional filtering and sorting.

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
