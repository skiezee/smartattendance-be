# Patrol API Documentation

## Base URL
```
http://localhost:8080/api
```

## Endpoints

### 1. Submit Patrol Incident
Submit a new patrol incident report with location, timestamp, and optional photo evidence.

**Endpoint:** `POST /patrol/incident`

**Request Body:**
```json
{
  "nik": "12345",
  "title": "Pintu Rusak",
  "description": "Pintu gudang B rusak dan tidak bisa dikunci",
  "latitude": -6.2088,
  "longitude": 106.8456,
  "timestamp": "22:30",
  "photo_base64": "data:image/jpeg;base64,/9j/4AAQSkZJRg..." // Optional
}
```

**Response (Success):**
```json
{
  "status": "success",
  "message": "Incident reported successfully",
  "incident_id": "patrol_incidents:abc123"
}
```

**Response (Error - Employee Not Found):**
```json
{
  "status": "error",
  "message": "Employee not found"
}
```

**Response (Error - Database Error):**
```json
{
  "status": "error",
  "message": "Database query error: ..."
}
```

---

### 2. Get All Patrol Incidents
Retrieve all patrol incidents from all employees, ordered by creation date (newest first).

**Endpoint:** `GET /patrol/incidents`

**Response (Success):**
```json
{
  "status": "success",
  "data": [
    {
      "id": {
        "tb": "patrol_incidents",
        "id": {
          "String": "abc123"
        }
      },
      "employee_id": {
        "tb": "employee",
        "id": {
          "String": "emp001"
        }
      },
      "nik": "12345",
      "title": "Pintu Rusak",
      "description": "Pintu gudang B rusak dan tidak bisa dikunci",
      "latitude": -6.2088,
      "longitude": 106.8456,
      "timestamp": "22:30",
      "photo_base64": "data:image/jpeg;base64,/9j/4AAQSkZJRg...",
      "created_at": "2026-04-15T22:30:00+07:00"
    },
    {
      "id": {
        "tb": "patrol_incidents",
        "id": {
          "String": "def456"
        }
      },
      "employee_id": {
        "tb": "employee",
        "id": {
          "String": "emp001"
        }
      },
      "nik": "12345",
      "title": "Lampu Mati",
      "description": "Lampu di area parkir C mati",
      "latitude": -6.2085,
      "longitude": 106.8450,
      "timestamp": "21:45",
      "photo_base64": null,
      "created_at": "2026-04-15T21:45:00+07:00"
    }
  ]
}
```

**Response (Error):**
```json
{
  "status": "error",
  "message": "Database query error: ..."
}
```

---

### 3. Get Patrol Incidents by NIK
Retrieve all patrol incidents for a specific employee by their NIK, ordered by creation date (newest first).

**Endpoint:** `GET /patrol/incidents/{nik}`

**Path Parameters:**
- `nik` (string, required): Employee NIK

**Example:** `GET /patrol/incidents/12345`

**Response (Success):**
```json
{
  "status": "success",
  "data": [
    {
      "id": {
        "tb": "patrol_incidents",
        "id": {
          "String": "abc123"
        }
      },
      "employee_id": {
        "tb": "employee",
        "id": {
          "String": "emp001"
        }
      },
      "nik": "12345",
      "title": "Pintu Rusak",
      "description": "Pintu gudang B rusak dan tidak bisa dikunci",
      "latitude": -6.2088,
      "longitude": 106.8456,
      "timestamp": "22:30",
      "photo_base64": "data:image/jpeg;base64,/9j/4AAQSkZJRg...",
      "created_at": "2026-04-15T22:30:00+07:00"
    }
  ]
}
```

**Response (Error):**
```json
{
  "status": "error",
  "message": "Database query error: ..."
}
```

---

## Data Models

### PatrolIncidentRequest
```rust
{
  nik: String,              // Employee NIK
  title: String,            // Incident title
  description: String,      // Incident description
  latitude: f64,            // GPS latitude
  longitude: f64,           // GPS longitude
  timestamp: String,        // Time of incident (HH:mm format)
  photo_base64: Option<String>  // Optional base64 encoded photo
}
```

### PatrolIncident (Database Record)
```rust
{
  id: Option<Thing>,        // SurrealDB record ID
  employee_id: Thing,       // Reference to employee record
  nik: String,              // Employee NIK
  title: String,            // Incident title
  description: String,      // Incident description
  latitude: f64,            // GPS latitude
  longitude: f64,           // GPS longitude
  timestamp: String,        // Time of incident
  photo_base64: Option<String>,  // Base64 encoded photo
  created_at: String        // ISO 8601 timestamp
}
```

---

## Database Schema

### Table: `patrol_incidents`

The patrol incidents are stored in SurrealDB with the following structure:

```sql
DEFINE TABLE patrol_incidents SCHEMAFULL;

DEFINE FIELD employee_id ON patrol_incidents TYPE record(employee);
DEFINE FIELD nik ON patrol_incidents TYPE string;
DEFINE FIELD title ON patrol_incidents TYPE string;
DEFINE FIELD description ON patrol_incidents TYPE string;
DEFINE FIELD latitude ON patrol_incidents TYPE float;
DEFINE FIELD longitude ON patrol_incidents TYPE float;
DEFINE FIELD timestamp ON patrol_incidents TYPE string;
DEFINE FIELD photo_base64 ON patrol_incidents TYPE option<string>;
DEFINE FIELD created_at ON patrol_incidents TYPE string;

DEFINE INDEX idx_nik ON patrol_incidents FIELDS nik;
DEFINE INDEX idx_created_at ON patrol_incidents FIELDS created_at;
```

---

## Testing with cURL

### Submit Incident
```bash
curl -X POST http://localhost:8080/api/patrol/incident \
  -H "Content-Type: application/json" \
  -d '{
    "nik": "12345",
    "title": "Pintu Rusak",
    "description": "Pintu gudang B rusak dan tidak bisa dikunci",
    "latitude": -6.2088,
    "longitude": 106.8456,
    "timestamp": "22:30",
    "photo_base64": null
  }'
```

### Get All Incidents
```bash
curl -X GET http://localhost:8080/api/patrol/incidents
```

### Get Incidents by NIK
```bash
curl -X GET http://localhost:8080/api/patrol/incidents/12345
```

---

## Error Handling

All endpoints follow a consistent error response format:

```json
{
  "status": "error",
  "message": "Error description here"
}
```

Common error scenarios:
- **Employee not found**: NIK doesn't exist in the database
- **Database query error**: SurrealDB connection or query issues
- **Failed to parse**: Data serialization/deserialization errors

---

## Notes

1. **Photo Storage**: Photos are stored as base64 encoded strings in the database. For production, consider using external storage (S3, Cloud Storage) and storing only URLs.

2. **Timestamp Format**: The `timestamp` field uses HH:mm format (e.g., "22:30"). The `created_at` field uses ISO 8601 format with timezone.

3. **GPS Coordinates**: Latitude and longitude are stored as f64 (double precision floating point) for accuracy.

4. **Employee Validation**: The API validates that the employee exists before creating an incident record.

5. **Ordering**: All list endpoints return incidents ordered by `created_at` in descending order (newest first).

---

## Integration with Android App

The Android app should:

1. Convert photo URI to base64 string before sending
2. Get current GPS coordinates from LocationManager
3. Format timestamp as HH:mm
4. Include employee NIK from session
5. Handle success/error responses appropriately

Example Kotlin code:
```kotlin
data class IncidentRequest(
    val nik: String,
    val title: String,
    val description: String,
    val latitude: Double,
    val longitude: Double,
    val timestamp: String,
    val photo_base64: String?
)

suspend fun submitIncident(incident: IncidentRequest): Result<IncidentResponse> {
    return try {
        val response = httpClient.post("http://localhost:8080/api/patrol/incident") {
            contentType(ContentType.Application.Json)
            setBody(incident)
        }
        Result.success(response.body())
    } catch (e: Exception) {
        Result.failure(e)
    }
}
```
