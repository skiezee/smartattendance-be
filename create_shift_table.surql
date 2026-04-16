-- Create shift_schedules table
-- Run this script in SurrealDB to create the shift schedules table

-- Define the table (SCHEMALESS allows flexible structure)
DEFINE TABLE shift_schedules SCHEMALESS;

-- Create indexes for better query performance
DEFINE INDEX idx_nik ON shift_schedules FIELDS nik;
DEFINE INDEX idx_date ON shift_schedules FIELDS date;
DEFINE INDEX idx_status ON shift_schedules FIELDS status;

-- Display confirmation
INFO FOR TABLE shift_schedules;
