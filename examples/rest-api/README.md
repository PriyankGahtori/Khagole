# Khagole REST API Example — Vedic Astrology Microservice

A high-performance REST API built with FastAPI and `khagole-engine` on NASA JPL DE440s ephemerides.

## Installation

```bash
pip install fastapi uvicorn khagole-engine
```

## Running the Server

```bash
uvicorn app:app --host 0.0.0.0 --port 8000 --reload
```

Interactive OpenAPI Swagger UI is available at:
👉 **`http://localhost:8000/docs`**

## Example Endpoints & Curl Commands

### 1. Health Check
```bash
curl -X GET "http://localhost:8000/health"
```

### 2. Celebrity Horoscope (Steve Jobs, Albert Einstein, Narendra Modi)
```bash
curl -X GET "http://localhost:8000/api/celebrity/jobs"
```

Returns the complete horoscope with Navagraha positions, Nakshatras & Padas, Panchanga, D9 Navamsha, and Vimshottari Dasha.

### 3. Calculate Custom Birth Chart
```bash
curl -X POST "http://localhost:8000/api/chart" \
  -H "Content-Type: application/json" \
  -d '{
    "utc": "1955-02-25T03:15:00Z",
    "latitude": 37.7749,
    "longitude": -122.4194,
    "zodiac_type": "sidereal",
    "ayanamsa": "lahiri",
    "house_system": "W",
    "position_mode": "true",
    "node_type": "mean"
  }'
```

### 4. Calculate Vedic Panchanga
```bash
curl -X POST "http://localhost:8000/api/panchang" \
  -H "Content-Type: application/json" \
  -d '{
    "sun_longitude": 150.5392,
    "moon_longitude": 217.9496,
    "utc": "1950-09-17T04:04:00Z"
  }'
```

### 5. Calculate Vimshottari Dasha
```bash
curl -X POST "http://localhost:8000/api/dasha" \
  -H "Content-Type: application/json" \
  -d '{
    "moon_longitude": 217.9496,
    "utc": "1950-09-17T04:04:00Z",
    "depth": 2,
    "year_type": "tssy"
  }'
```
