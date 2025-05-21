# Flight API Documentation

This document provides information about the Flight API endpoints, including request parameters, authentication requirements, and example usage.

## Base URL

All endpoints are relative to the base URL of the API server.

## Authentication

The `/flights` POST endpoint requires password authentication via query parameter.

## Endpoints

### Get Flights

Retrieves flight information with optional filtering, sorting, and pagination.

```
GET /flights
```

#### Query Parameters

| Parameter | Type | Description | Default |
|-----------|------|-------------|---------|
| `origin` | string | Filter flights by origin location | None |
| `destination` | string | Filter flights by destination location | None |
| `max_price` | number | Maximum price in INR | None |
| `max_rain` | number | Maximum rain probability (0-100) | None |
| `airline` | string | Comma-separated list of airlines to filter by | None |
| `sort_by` | string | Sort results by ("date" or "price") | "price" |
| `page` | number | Page number for pagination | 1 |
| `limit` | number | Number of items per page (max 20) | 20 |

#### Response Format

```json
{
  "data": [
    {
      "id": 1,
      "airline": "IndiGo",
      "flight_number": "6E123",
      "origin": "Mumbai",
      "destination": "Delhi",
      "date": "2025-05-25",
      "departure_time": "10:00",
      "arrival_time": "12:00",
      "price_inr": 5000,
      "available_seats": 45,
      "rain_probability": 30
    },
    // Additional flight objects...
  ],
  "page": 1,
  "total_pages": 5,
  "total_items": 100
}
```

#### Example Requests

1. Basic flight search:
```
GET /flights
```

2. Filtered search:
```
GET /flights?origin=Mumbai&destination=Delhi&max_price=6000
```

3. Airline-specific search with pagination:
```
GET /flights?airline=IndiGo,AirIndia&page=2&limit=10
```

4. Filter by rain probability and sort by date:
```
GET /flights?max_rain=20&sort_by=date
```

### Insert Flights

Replaces all existing flights with a new set of flights.

```
POST /flights?password={password}
```

#### Authentication

This endpoint requires authentication using the `password` query parameter. The password must match the value stored in the `FLIGHT_API_PASSWORD` environment variable.

#### Request Body

An array of flight objects in the following format:

```json
[
  {
    "airline": "IndiGo",
    "flight_number": "6E123",
    "origin": "Mumbai",
    "destination": "Delhi",
    "date": "2025-05-25",
    "departure_time": "10:00",
    "arrival_time": "12:00",
    "price_inr": 5000,
    "available_seats": 45,
    "rain_probability": 30
  },
  // Additional flight objects...
]
```

#### Response Format

```json
{
  "status": "success",
  "message": "Successfully deleted all flights and inserted X new flights"
}
```

#### Example Request

```bash
curl -X POST \
  -H "Content-Type: application/json" \
  -d '[{"airline":"IndiGo","flight_number":"6E123","origin":"Mumbai","destination":"Delhi","date":"2025-05-25","departure_time":"10:00","arrival_time":"12:00","price_inr":5000,"available_seats":45,"rain_probability":30}]' \
  'http://api.example.com/flights?password=your_password_here'
```

## Error Responses

| Status Code | Description |
|-------------|-------------|
| 401 | Unauthorized - Invalid or missing password |
| 500 | Internal Server Error - Database or server issue |
| 503 | Service Unavailable - Database connection issue |

## Notes

1. This API includes weather information through the `rain_probability` field, which indicates the chance of rain at the destination (value between 0-100).
2. All prices are in Indian Rupees (INR).
3. The maximum number of results per page is capped at 20, even if a larger value is requested.
4. When inserting flights, all existing flight data is deleted before the new flights are added.
