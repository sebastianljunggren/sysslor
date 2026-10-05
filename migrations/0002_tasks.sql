-- Timestamps are UTC unix milliseconds.

CREATE TABLE executors (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL
) STRICT;

CREATE TABLE tasks (
    id INTEGER PRIMARY KEY,
    project_id INTEGER NOT NULL REFERENCES projects (id),
    name TEXT NOT NULL,
    cadence_amount INTEGER NOT NULL CHECK (cadence_amount >= 1),
    cadence_unit TEXT NOT NULL CHECK (cadence_unit IN ('days', 'weeks')),
    priority INTEGER NOT NULL CHECK (priority BETWEEN 0 AND 5),
    created_at INTEGER NOT NULL,
    archived_at INTEGER
) STRICT;

CREATE INDEX tasks_project_id ON tasks (project_id);

CREATE TABLE completions (
    -- Client-generated UUIDv7, hyphenated lowercase.
    id TEXT PRIMARY KEY,
    task_id INTEGER NOT NULL REFERENCES tasks (id),
    executor_id INTEGER NOT NULL REFERENCES executors (id),
    completed_at INTEGER NOT NULL
) STRICT;

CREATE INDEX completions_task_id_completed_at ON completions (task_id, completed_at);
