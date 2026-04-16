-- Insert sample shift schedules
-- Run this after creating the shift_schedules table

-- Sample shifts for employee with NIK 12345
CREATE shift_schedules CONTENT {
    employee_id: employee:emp001,
    nik: "12345",
    employee_name: "John Doe",
    shift_type: "PAGI",
    date: "2026-04-20",
    start_time: "06:00",
    end_time: "14:00",
    location: "Gedung A - Lantai 1",
    tasks: ["Patroli area 1", "Check keamanan", "Laporan harian"],
    status: "SCHEDULED",
    notes: "Perhatian khusus area parkir",
    created_at: time::now()
};

CREATE shift_schedules CONTENT {
    employee_id: employee:emp001,
    nik: "12345",
    employee_name: "John Doe",
    shift_type: "SIANG",
    date: "2026-04-21",
    start_time: "14:00",
    end_time: "22:00",
    location: "Gedung A - Lantai 2",
    tasks: ["Patroli area 2", "Check keamanan", "Koordinasi tim"],
    status: "SCHEDULED",
    notes: NONE,
    created_at: time::now()
};

CREATE shift_schedules CONTENT {
    employee_id: employee:emp001,
    nik: "12345",
    employee_name: "John Doe",
    shift_type: "MALAM",
    date: "2026-04-22",
    start_time: "22:00",
    end_time: "06:00",
    location: "Gedung B - Lantai 1",
    tasks: ["Patroli malam", "Monitoring CCTV", "Laporan insiden"],
    status: "SCHEDULED",
    notes: "Shift malam memerlukan koordinasi dengan security",
    created_at: time::now()
};

-- Display all shifts
SELECT * FROM shift_schedules ORDER BY date ASC;
