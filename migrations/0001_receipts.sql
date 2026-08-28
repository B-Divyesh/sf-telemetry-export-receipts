CREATE TABLE IF NOT EXISTS receipts (
    id TEXT PRIMARY KEY NOT NULL,
    created_at TEXT NOT NULL,
    requester TEXT NOT NULL,
    endpoint TEXT NOT NULL,
    outcome TEXT NOT NULL,
    receipt_json TEXT NOT NULL,
    signature TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS receipts_created_at ON receipts(created_at DESC);
CREATE INDEX IF NOT EXISTS receipts_requester ON receipts(requester);
CREATE INDEX IF NOT EXISTS receipts_outcome ON receipts(outcome);

