-- Optional grouping of tasks within a project, e.g. rooms in a home.
-- Named task_groups because GROUPS is an SQLite keyword.
CREATE TABLE task_groups (
    id INTEGER PRIMARY KEY,
    project_id INTEGER NOT NULL REFERENCES projects (id),
    name TEXT NOT NULL
) STRICT;
CREATE INDEX task_groups_project_id ON task_groups (project_id);

ALTER TABLE tasks ADD COLUMN group_id INTEGER REFERENCES task_groups (id);
CREATE INDEX tasks_group_id ON tasks (group_id);
