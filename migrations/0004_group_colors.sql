-- Optional accent for a group's tag. NULL means the neutral border.
ALTER TABLE task_groups ADD COLUMN color TEXT
    CHECK (color IN ('yellow', 'orange', 'red', 'magenta', 'violet', 'blue', 'cyan', 'green'));
