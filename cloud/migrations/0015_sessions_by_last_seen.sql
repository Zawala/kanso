-- Server metrics (docs/METRICS.md › Server metrics): `devices_seen` counts the sessions seen in
-- one hour. Without this index that count would scan every session; with it, only the hour's.
CREATE INDEX sessions_by_last_seen ON sessions (last_seen_at);
