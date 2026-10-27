-- Crash reports from the mobile apps (docs/CRASH-REPORTS.md): the Worker validates a report
-- against the page's patterns and stores the object it validated, re-serialized, so `report` is
-- canonical JSON and never the raw body. A report holds no account, device ID or IP address, and
-- the Worker reads none of them: the sender is only counted by the `limits` table (a keyed hash of
-- its IP group that expires with its window).
--
-- received_day is the UTC day the Worker received the report (not the report's own `day`).
-- kind, type and app_version repeat fields of `report` for querying. The cron deletes rows older
-- than 90 days.
CREATE TABLE crash_reports (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    received_day TEXT NOT NULL CHECK (received_day GLOB '[0-9][0-9][0-9][0-9]-[0-9][0-9]-[0-9][0-9]'),
    kind TEXT NOT NULL CHECK (kind IN ('dart', 'ios-native', 'android-native', 'android-anr', 'android-jvm')),
    type TEXT NOT NULL CHECK (length(type) BETWEEN 1 AND 128),
    app_version TEXT NOT NULL CHECK (length(app_version) BETWEEN 5 AND 14),
    report TEXT NOT NULL CHECK (json_valid(report) AND length(report) <= 32768)
);
CREATE INDEX crash_reports_received ON crash_reports (received_day);
