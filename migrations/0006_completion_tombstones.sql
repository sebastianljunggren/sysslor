-- Ids of undone completions, so a PUT that arrives late (a retry, a timed-out request
-- still being processed, another device's offline queue) can't bring one back.
CREATE TABLE deleted_completions (
    id TEXT PRIMARY KEY,
    deleted_at INTEGER NOT NULL
);
