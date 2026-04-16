# Shift Schedule API Documentation

## Base URL
```
http://localhost:8080/api
```

## Endpoints

### 1. Create Shift Schedule
Create a new shift schedule for an employee.

**Endpoint:** `POST /shift`

**Request Body:**
```json
{
  "nik": "12345",
  "shift_type": "PAGI",
  "date": "2026-04-20",
  "start_time": "06:00",
  "end_time": "14:00",
  "location": "Gedung A - Lantai 1",
  "tasks": [
    "Patroli area 1",
    "Check keamanan",
    "Laporan harian"
  ],
  "notes": "Perhatian khusus area parkir"
}
```

**Response (Success):**
```json
{
  "status": "success",
  "message": "Shift schedule created successfully",
  "shift_id": "shift_schedules:abc123"
}
```

---

### 2. Get Shifts by NIK
Retrieve shift schedules for a specific employee with optional date filtering.

**Endpoint:** `POST /shift/list`

**Request Body:**
```json
{
  "nik": "12345",
  "start_date": "2026-04-01",  // Optional
  "end_date": "2026-04-30"     // Optional
}
```

**Response (Success):**
```json
{
  "status": "success",
  "data": [
    {
      "id": {
        "tb": "shift_schedules",
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
      "employee_name": "John Doe",
      "shift_type": "PAGI",
      "date": "2026-04-20",
      "start_time": "06:00",
      "end_time": "14:00",
      "location": "Gedung A - Lantai 1",
      "tasks": [
        "Patroli area 1",
        "Check keamanan",
        "Laporan harian"
      ],
      "status": "SCHEDULED",
      "notes": "Perhatian khusus area parkir",
      "created_at": "2026-04-15T10:00:00+07:00"
    }
  ]
}
```

---

### 3. Get All Shifts
Retrieve all shift schedules with optional date filtering.

**Endpoint:** `GET /shift/all?start_date=2026-04-01&end_date=2026-04-30`

**Query Parameters:**
- `start_date` (optional): Start date in YYYY-MM-DD format
- `end_date` (optional): End date in YYYY-MM-DD format

**Response (Success):**
```json
{
  "status": "success",
  "data": [
    {
      "id": {
        "tb": "shift_schedules",
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
      "employee_name": "John Doe",
      "shift_type": "PAGI",
      "date": "2026-04-20",
      "start_time": "06:00",
      "end_time": "14:00",
      "location": "Gedung A - Lantai 1",
      "tasks": [
        "Patroli area 1",
        "Check keamanan"
      ],
      "status": "SCHEDULED",
      "notes": null,
      "created_at": "2026-04-15T10:00:00+07:00"
    }
  ]
}
```

---

### 4. Update Shift Status
Update the status of a shift schedule.

**Endpoint:** `PUT /shift/status`

**Request Body:**
```json
{
  "shift_id": "shift_schedules:abc123",
  "status": "COMPLETED"
}
```

**Valid Status Values:**
- `SCHEDULED` - Shift is scheduled
- `COMPLETED` - Shift has been completed
- `CANCELLED` - Shift has been cancelled

**Response (Success):**
```json
{
  "status": "success",
  "message": "Shift status updated successfully",
  "shift_id": "shift_schedules:abc123"
}
```

---

### 5. Get Shift Statistics
Get statistics for an employee's shifts.

**Endpoint:** `GET /shift/stats/{nik}`

**Path Parameters:**
- `nik` (string, required): Employee NIK

**Example:** `GET /shift/stats/12345`

**Response (Success):**
```json
{
  "status": "success",
  "stats": {
    "total_shifts": 30,
    "completed_shifts": 25,
    "upcoming_shifts": 4,
    "cancelled_shifts": 1
  }
}
```

---

### 6. Delete Shift
Delete a shift schedule.

**Endpoint:** `DELETE /shift/{shift_id}`

**Path Parameters:**
- `shift_id` (string, required): Shift ID (e.g., "shift_schedules:abc123")

**Example:** `DELETE /shift/shift_schedules:abc123`

**Response (Success):**
```json
{
  "status": "success",
  "message": "Shift deleted successfully",
  "shift_id": "shift_schedules:abc123"
}
```

---

## Data Models

### CreateShiftRequest
```rust
{
  nik: String,              // Employee NIK
  shift_type: String,       // "PAGI", "SIANG", "MALAM"
  date: String,             // YYYY-MM-DD format
  start_time: String,       // HH:mm format
  end_time: String,         // HH:mm format
  location: String,         // Shift location
  tasks: Vec<String>,       // List of tasks
  notes: Option<String>     // Optional notes
}
```

### ShiftSchedule (Database Record)
```rust
{
  id: Option<Thing>,        // SurrealDB record ID
  employee_id: Thing,       // Reference to employee record
  nik: String,              // Employee NIK
  employee_name: String,    // Employee full name
  shift_type: String,       // "PAGI", "SIANG", "MALAM"
  date: String,             // YYYY-MM-DD
  start_time: String,       // HH:mm
  end_time: String,         // HH:mm
  location: String,         // Shift location
  tasks: Vec<String>,       // List of tasks
  status: String,           // "SCHEDULED", "COMPLETED", "CANCELLED"
  notes: Option<String>,    // Optional notes
  created_at: String        // ISO 8601 timestamp
}
```

---

## Database Schema

### Table: `shift_schedules`

```sql
DEFINE TABLE shift_schedules SCHEMALESS;

DEFINE INDEX idx_nik ON shift_schedules FIELDS nik;
DEFINE INDEX idx_date ON shift_schedules FIELDS date;
DEFINE INDEX idx_status ON shift_schedules FIELDS status;
```

---

## Testing with cURL

### Create Shift
```bash
curl -X POST http://localhost:8080/api/shift \
  -H "Content-Type: application/json" \
  -d '{
    "nik": "12345",
    "shift_type": "PAGI",
    "date": "2026-04-20",
    "start_time": "06:00",
    "end_time": "14:00",
    "location": "Gedung A - Lantai 1",
    "tasks": ["Patroli area 1", "Check keamanan"],
    "notes": "Perhatian khusus area parkir"
  }'
```

### Get Shifts by NIK
```bash
curl -X POST http://localhost:8080/api/shift/list \
  -H "Content-Type: application/json" \
  -d '{
    "nik": "12345",
    "start_date": "2026-04-01",
    "end_date": "2026-04-30"
  }'
```

### Get All Shifts
```bash
curl -X GET "http://localhost:8080/api/shift/all?start_date=2026-04-01&end_date=2026-04-30"
```

### Update Shift Status
```bash
curl -X PUT http://localhost:8080/api/shift/status \
  -H "Content-Type: application/json" \
  -d '{
    "shift_id": "shift_schedules:abc123",
    "status": "COMPLETED"
  }'
```

### Get Shift Stats
```bash
curl -X GET http://localhost:8080/api/shift/stats/12345
```

### Delete Shift
```bash
curl -X DELETE http://localhost:8080/api/shift/shift_schedules:abc123
```

---

## Shift Types

| Type | Start Time | End Time | Description |
|------|------------|----------|-------------|
| PAGI | 06:00 | 14:00 | Morning shift |
| SIANG | 14:00 | 22:00 | Afternoon shift |
| MALAM | 22:00 | 06:00 | Night shift |

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
- **Invalid date format**: Date must be in YYYY-MM-DD format
- **Invalid time format**: Time must be in HH:mm format

---

## Notes

1. **Date Format**: All dates use YYYY-MM-DD format (ISO 8601)
2. **Time Format**: All times use HH:mm format (24-hour)
3. **Shift Types**: Currently supports PAGI, SIANG, MALAM
4. **Employee Validation**: API validates employee exists before creating shift
5. **Ordering**: List endpoints return shifts ordered by date and start_time
6. **Filtering**: Date filtering is inclusive (includes both start_date and end_date)

---

## Integration with Android App

The Android app should:

1. Fetch shifts on screen load
2. Filter by week/month view
3. Display shift cards with color coding by shift type
4. Show shift details in dialog
5. Handle loading and error states
6. Refresh data periodically

Example Kotlin code:
```kotlin
data class ShiftRequest(
    val nik: String,
    val startDate: String?,
    val endDate: String?
)

suspend fun getShifts(request: ShiftRequest): Result<ShiftListResponse> {
    return try {
        val response = httpClient.post("http://localhost:8080/api/shift/list") {
            contentType(ContentType.Application.Json)
            setBody(request)
        }
        Result.success(response.body())
    } catch (e: Exception) {
        Result.failure(e)
    }
}
```
