-- Fix existing employees that don't have attendance_requirement
-- This script will add default attendance_requirement to all employees

-- Update all employees without attendance_requirement
UPDATE employee SET 
    attendance_requirement = {
        wifi_enabled: false,
        wifi_ssids: NONE,
        location_enabled: false,
        location_boundaries: NONE,
        face_recognition_enabled: false,
        fingerprint_enabled: false
    }
WHERE attendance_requirement = NONE OR attendance_requirement = NULL;

-- Verify the update
SELECT nik, full_name, department, status, attendance_requirement FROM employee LIMIT 10;
