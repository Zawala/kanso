-- Server metrics (docs/METRICS.md › Server metrics): request counts as aggregates only. One row
-- per 15-minute window, route group (a name from the router, never a path), status class and
-- latency bucket. No IP, account, device or token is ever stored here. The Worker adds to the
-- rows in batches (src/server_stats.ts) and the cron reads a finished window, sends it as one
-- `server_stats` event and deletes it and everything older.
CREATE TABLE request_counts (
    window_start INTEGER NOT NULL,
    route_group TEXT NOT NULL,
    status_class TEXT NOT NULL CHECK (status_class IN ('2xx', '3xx', '4xx', '5xx')),
    latency_bucket TEXT NOT NULL,
    count INTEGER NOT NULL CHECK (count >= 0),
    PRIMARY KEY (window_start, route_group, status_class, latency_bucket)
) WITHOUT ROWID;
