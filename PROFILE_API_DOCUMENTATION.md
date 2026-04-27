# Profile API Documentation

## Overview

API endpoints untuk manajemen profile karyawan termasuk get profile, update profile, change password, dan upload photo.

## Base URL

```
http://192.168.100.132:8080/api
```

## Endpoints

### 1. Get Profile

Mendapatkan detail profile karyawan beserta statistik attendance.

**Endpoint:** `GET /profile/{nik}`

**Parameters:**
- `nik` (path parameter): NIK karyawan

**Response Success (200):**
```json
{
  "status": "success",
  "data": {
    "nik": "EMP001",
    "full_name": "John Doe",
    "email": "john@example.com",
    "phone_number": "+62812345678",
    "department": "IT Department",
    "position": "Software Engineer",
    "join_date": "2024-01-01T00:00:00Z",
    "profile_photo_url": "/uploads/profile_photos/profile_EMP001_uuid.jpg",
    "present_count": 24,
    "absent_count": 2,
    "leave_count": 3
  }
}
```

**Response Error (404):**
```json
{
  "status": "error",
  "message": "Employee not found"
}
```

**Example Request:**
```bash
curl -X GET http://192.168.100.132:8080/api/profile/EMP001
```

---

### 2. Update Profile

Update informasi profile karyawan (nama, email, phone, photo URL).

**Endpoint:** `PUT /profile`

**Request Body:**
```json
{
  "nik": "EMP001",
  "full_name": "John Doe Updated",
  "email": "john.new@example.com",
  "phone_number": "+62812345679",
  "profile_photo_url": "/uploads/profile_photos/profile_EMP001_uuid.jpg"
}
```

**Fields:**
- `nik` (required): NIK karyawan
- `full_name` (optional): Nama lengkap baru
- `email` (optional): Email baru
- `phone_number` (optional): Nomor telepon baru
- `profile_photo_url` (optional): URL foto profile baru

**Response Success (200):**
```json
{
  "status": "success",
  "message": "Profile updated successfully"
}
```

**Response Error (404):**
```json
{
  "status": "error",
  "message": "Employee not found"
}
```

**Response Error (400):**
```json
{
  "status": "error",
  "message": "No fields to update"
}
```

**Example Request:**
```bash
curl -X PUT http://192.168.100.132:8080/api/profile \
  -H "Content-Type: application/json" \
  -d '{
    "nik": "EMP001",
    "full_name": "John Doe Updated",
    "email": "john.new@example.com",
    "phone_number": "+62812345679"
  }'
```

---

### 3. Change Password

Mengubah password karyawan dengan verifikasi password lama.

**Endpoint:** `POST /profile/change-password`

**Request Body:**
```json
{
  "nik": "EMP001",
  "old_password": "oldpassword123",
  "new_password": "newpassword456"
}
```

**Fields:**
- `nik` (required): NIK karyawan
- `old_password` (required): Password lama untuk verifikasi
- `new_password` (required): Password baru

**Response Success (200):**
```json
{
  "status": "success",
  "message": "Password changed successfully"
}
```

**Response Error (401):**
```json
{
  "status": "error",
  "message": "Old password is incorrect"
}
```

**Response Error (404):**
```json
{
  "status": "error",
  "message": "Employee not found"
}
```

**Example Request:**
```bash
curl -X POST http://192.168.100.132:8080/api/profile/change-password \
  -H "Content-Type: application/json" \
  -d '{
    "nik": "EMP001",
    "old_password": "oldpassword123",
    "new_password": "newpassword456"
  }'
```

---

### 4. Upload Profile Photo

Upload foto profile karyawan (multipart/form-data).

**Endpoint:** `POST /profile/upload-photo`

**Content-Type:** `multipart/form-data`

**Form Fields:**
- `nik` (required): NIK karyawan
- `photo` (required): File foto (image/*)

**Response Success (200):**
```json
{
  "status": "success",
  "message": "Profile photo uploaded successfully",
  "data": {
    "photo_url": "/uploads/profile_photos/profile_EMP001_uuid.jpg"
  }
}
```

**Response Error (404):**
```json
{
  "status": "error",
  "message": "Employee not found"
}
```

**Response Error (400):**
```json
{
  "status": "error",
  "message": "NIK is required"
}
```

**Example Request (curl):**
```bash
curl -X POST http://192.168.100.132:8080/api/profile/upload-photo \
  -F "nik=EMP001" \
  -F "photo=@/path/to/photo.jpg"
```

**Example Request (JavaScript):**
```javascript
const formData = new FormData();
formData.append('nik', 'EMP001');
formData.append('photo', fileInput.files[0]);

fetch('http://192.168.100.132:8080/api/profile/upload-photo', {
  method: 'POST',
  body: formData
})
.then(response => response.json())
.then(data => console.log(data));
```

---

## File Storage

### Upload Directory

Foto profile disimpan di:
```
smartattendance_be/uploads/profile_photos/
```

### File Naming Convention

Format nama file:
```
profile_{NIK}_{UUID}.{extension}
```

Contoh:
```
profile_EMP001_550e8400-e29b-41d4-a716-446655440000.jpg
```

### Accessing Uploaded Files

File dapat diakses melalui URL:
```
http://192.168.100.132:8080/uploads/profile_photos/profile_EMP001_uuid.jpg
```

---

## Error Codes

| Status Code | Description |
|-------------|-------------|
| 200 | Success |
| 400 | Bad Request (missing fields, invalid data) |
| 401 | Unauthorized (wrong password) |
| 404 | Not Found (employee not found) |
| 500 | Internal Server Error |

---

## Security Notes

1. **Password Hashing**: Password di-hash menggunakan bcrypt dengan DEFAULT_COST
2. **Password Verification**: Old password diverifikasi sebelum update
3. **File Upload**: File disimpan dengan UUID untuk menghindari collision
4. **SQL Injection**: Menggunakan parameterized queries
5. **Input Sanitization**: Single quotes di-escape untuk mencegah injection

---

## Database Schema

### Employee Table Fields

```sql
employee {
  id: Thing,
  nik: String,
  full_name: String,
  email: String,
  phone_number: String (optional),
  password_hash: String,
  role: String,
  department_id: Thing (optional),
  position_id: Thing (optional),
  employment_status: String,
  profile_photo_url: String (optional),
  join_date: DateTime,
  attendance_requirement: Object,
  created_at: DateTime,
  updated_at: DateTime
}
```

---

## Integration with Mobile App

### ProfileService.kt

```kotlin
object ProfileService {
    suspend fun getProfile(nik: String): Result<ProfileDetailResponse> {
        // GET /api/profile/{nik}
    }
    
    suspend fun updateProfile(request: UpdateProfileRequest): Result<String> {
        // PUT /api/profile
    }
    
    suspend fun changePassword(request: ChangePasswordRequest): Result<String> {
        // POST /api/profile/change-password
    }
}

object PhotoUploadService {
    suspend fun uploadPhoto(
        context: Context,
        uri: Uri,
        nik: String
    ): Result<String> {
        // POST /api/profile/upload-photo (multipart)
    }
}
```

### Flow Diagram

```
Mobile App                    Backend API                   Database
    |                             |                             |
    |-- GET /profile/{nik} ------>|                             |
    |                             |-- SELECT employee --------->|
    |                             |<-- employee data -----------|
    |                             |-- SELECT attendance ------->|
    |                             |<-- attendance stats --------|
    |<-- profile + stats ---------|                             |
    |                             |                             |
    |-- PUT /profile ------------>|                             |
    |                             |-- UPDATE employee --------->|
    |<-- success message ---------|<-- updated ----------------|
    |                             |                             |
    |-- POST /change-password --->|                             |
    |                             |-- SELECT password_hash ---->|
    |                             |<-- current hash ------------|
    |                             |-- verify old password       |
    |                             |-- hash new password         |
    |                             |-- UPDATE password_hash ---->|
    |<-- success message ---------|<-- updated ----------------|
    |                             |                             |
    |-- POST /upload-photo ------>|                             |
    |   (multipart)               |-- save file to disk         |
    |                             |-- UPDATE profile_photo_url->|
    |<-- photo_url --------------|<-- updated ----------------|
```

---

## Testing

### Test Get Profile
```bash
curl -X GET http://192.168.100.132:8080/api/profile/EMP001
```

### Test Update Profile
```bash
curl -X PUT http://192.168.100.132:8080/api/profile \
  -H "Content-Type: application/json" \
  -d '{"nik":"EMP001","full_name":"Test User","email":"test@example.com"}'
```

### Test Change Password
```bash
curl -X POST http://192.168.100.132:8080/api/profile/change-password \
  -H "Content-Type: application/json" \
  -d '{"nik":"EMP001","old_password":"password123","new_password":"newpass456"}'
```

### Test Upload Photo
```bash
curl -X POST http://192.168.100.132:8080/api/profile/upload-photo \
  -F "nik=EMP001" \
  -F "photo=@test_photo.jpg"
```

---

## Changelog

### Version 1.0.0 (2026-04-27)
- Initial release
- Added GET /profile/{nik}
- Added PUT /profile
- Added POST /profile/change-password
- Added POST /profile/upload-photo
- Added static file serving for uploads directory

---

## Support

For issues or questions, contact:
- Email: support@smartattendance.com
- Phone: +62 812-3456-7890
