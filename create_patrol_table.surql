-- Create patrol_incidents table
-- Run this script in SurrealDB to create the patrol incidents table

-- Define the table (SCHEMALESS allows flexible structure)
DEFINE TABLE patrol_incidents SCHEMALESS;

-- Create indexes for better query performance
DEFINE INDEX idx_nik ON patrol_incidents FIELDS nik;
DEFINE INDEX idx_created_at ON patrol_incidents FIELDS created_at;

-- Display confirmation
INFO FOR TABLE patrol_incidents;
