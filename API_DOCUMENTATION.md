# Smart Attendance API Documentation

**Base URL:** `http://<IP_BACKEND>:8080/api`  
**Format:** JSON (`application/json`)

---

## 🔐 1. Authentication (Auth)

### 1.1 Login
Endpoint untuk masuk ke dalam aplikasi. Akan mereturn status berhasil dan menyimpan token FCM untuk push notification jika diberikan.

* **URL:** `/login`
* **Method:** `POST`
* **Request Body:**
  ```json
  {
    "nik": "1003",
    "password": "123",
    "fcm_token": "token_firebase_opsional"
  }
  ```
* **Response (Success 200 OK):**
  ```json
  {
    "status": "success",
    "message": "Welcome back, Budi"
  }
  ```
* **Response (Error 401/500):**
  ```json
  "Invalid NIK or password"
  ```

### 1.2 Register
Mendaftarkan karyawan baru ke dalam database.

* **URL:** `/register`
* **Method:** `POST`
* **Request Body:**
  ```json
  {
    "nik": "1004",
    "full_name": "Andi Setiawan",
    "email": "andi@perusahaan.com",
    "password": "password123"
  }
  ```
* **Response (Success 200 OK):**
  ```json
  {
    "status": "success",
    "message": "User registered successfully"
  }
  ```

---

## 📍 2. Attendance (Absensi)

### 2.1 Check Enrollment Status
Mengecek apakah karyawan sudah mendaftarkan wajah dan sidik jari, serta apakah sudah absen hari ini.

* **URL:** `/attendance/enrollment-status`
* **Method:** `POST`
* **Request Body:**
  ```json
  {
    "nik": "1003"
  }
  ```
* **Response (Success 200 OK):**
  ```json
  {
    "status": "success",
    "face_enrolled": true,
    "fingerprint_enrolled": true,
    "has_attended_today": false
  }
  ```

### 2.2 Enroll Face (Daftar Wajah)
Menyimpan data embedding (matriks) wajah karyawan.

* **URL:** `/attendance/enroll-face`
* **Method:** `POST`
* **Request Body:**
  ```json
  {
    "nik": "1003",
    "face_embedding": [0.12, -0.45, 0.88] 
  }
  ```
* **Response:**
  ```json
  {
    "status": "success",
    "message": "Face enrolled successfully"
  }
  ```

### 2.3 Enroll Fingerprint (Daftar Sidik Jari)
Menandai bahwa karyawan telah berhasil mendaftarkan sidik jari di perangkatnya.

* **URL:** `/attendance/enroll-fingerprint`
* **Method:** `POST`
* **Request Body:**
  ```json
  {
    "nik": "1003"
  }
  ```
* **Response:**
  ```json
  {
    "status": "success",
    "message": "Fingerprint enrolled successfully"
  }
  ```

### 2.4 Clock In / Clock Out (Absen Masuk)
Mencatat waktu absen dengan validasi biometrik.

* **URL:** `/attendance/clock-in`
* **Method:** `POST`
* **Request Body:**
  ```json
  {
    "nik": "1003",
    "lat": -6.200000,
    "lng": 106.816666,
    "method": "face+fingerprint", 
    "face_confidence": 0.95
  }
  ```
* **Response:**
  ```json
  {
    "status": "success",
    "message": "Clock in successful"
  }
  ```

### 2.5 Get All Attendance Logs
Mengambil semua data history absensi karyawan (biasanya dipakai untuk halaman Web HRD).

* **URL:** `/attendance/logs`
* **Method:** `GET`
* **Response (Success 200 OK):**
  ```json
  {
    "status": "success",
    "data": [
      {
        "id": { "tb": "attendance_log", "id": { "String": "xyz" } },
        "employee_id": { "tb": "employee", "id": { "String": "abc" } },
        "date": "2026-03-22T00:00:00Z",
        "check_in": "2026-03-22T08:00:00Z",
        "check_out": "2026-03-22T17:00:00Z",
        "status": "present",
        "location": "Office"
      }
    ]
  }
  ```

---

## 👥 3. Employees (Karyawan)

### 3.1 Get All Employees
Mengambil daftar semua karyawan (untuk halaman Web HRD).

* **URL:** `/employees`
* **Method:** `GET`
* **Response (Success 200 OK):**
  ```json
  {
    "status": "success",
    "data": [
      {
        "id": { "tb": "employee", "id": { "String": "abc" } },
        "nik": "1003",
        "full_name": "Cessariana",
        "email": "cessa@perusahaan.com",
        "role": "employee",
        "department": "Engineering",
        "status": "Active" 
      }
    ]
  }
  ```

---

## 🏖️ 4. Leave (Cuti)

### 4.1 Submit Leave Request (Pengajuan Cuti)
Mengirimkan form pengajuan cuti baru.

* **URL:** `/leave`
* **Method:** `POST`
* **Request Body:**
  ```json
  {
    "nik": "1003",
    "leave_type": "Cuti Tahunan",
    "start_date": "2026-03-25",
    "end_date": "2026-03-26",
    "duration": 2,
    "reason": "Acara keluarga"
  }
  ```
* **Response:**
  ```json
  {
    "status": "success",
    "message": "Leave request submitted successfully"
  }
  ```

### 4.2 Get Leaves (Ambil Daftar Cuti)
Mengambil daftar pengajuan cuti. Bisa mengambil semua (untuk admin) atau spesifik by NIK (untuk user).

* **URL:** `/leave?nik=1003` 
* **Method:** `GET`
* **Response (Success 200 OK):**
  ```json
  {
    "status": "success",
    "data": [
      {
        "id": { "tb": "leaves", "id": { "String": "xyz" } },
        "nik": "1003",
        "leave_type": "Cuti Tahunan",
        "start_date": "2026-03-25",
        "end_date": "2026-03-26",
        "duration": 2,
        "reason": "Acara keluarga",
        "status": "PENDING", 
        "stage1_status": "WAITING", 
        "stage2_status": "WAITING", 
        "created_at": "2026-03-22T10:00:00Z"
      }
    ]
  }
  ```

### 4.3 Update Leave Status (Approval Cuti)
Mengupdate status persetujuan cuti (digunakan di Website). Jika status `stage2` (HRD) di-approve atau reject, akan mengirimkan FCM Push Notification ke device Android karyawan terkait.

* **URL:** `/leave/status`
* **Method:** `PUT`
* **Request Body:**
  ```json
  {
    "id": "leaves:h8k3n2m9", 
    "stage": 2,              
    "status": "APPROVED"     
  }
  ```
* **Response:**
  ```json
  {
    "status": "success",
    "message": "Leave status updated"
  }
  ```
