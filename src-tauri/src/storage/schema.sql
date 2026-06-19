CREATE TABLE IF NOT EXISTS projects (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL CHECK(length(trim(name)) > 0),
    sum_time_length INTEGER NOT NULL DEFAULT 0 CHECK(sum_time_length >= 0),
    start_date TEXT NULL CHECK(
        start_date IS NULL
        OR (length(start_date) = 10 AND substr(start_date, 5, 1) = '-' AND substr(start_date, 8, 1) = '-')
    ),
    end_date TEXT NULL CHECK(
        end_date IS NULL
        OR (length(end_date) = 10 AND substr(end_date, 5, 1) = '-' AND substr(end_date, 8, 1) = '-')
    ),
    is_done INTEGER NOT NULL DEFAULT 0 CHECK(is_done IN (0, 1)),
    project_category_id TEXT NULL CHECK(project_category_id IS NULL OR length(trim(project_category_id)) > 0)
);

CREATE TABLE IF NOT EXISTS tasks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    category_id TEXT NOT NULL CHECK(length(trim(category_id)) > 0),
    time_length INTEGER NOT NULL CHECK(time_length >= 0),
    date TEXT NOT NULL CHECK(
        length(date) = 10
        AND substr(date, 5, 1) = '-'
        AND substr(date, 8, 1) = '-'
    ),
    project_id INTEGER NULL REFERENCES projects(id) ON DELETE SET NULL,
    is_project_closing_task INTEGER NOT NULL DEFAULT 0 CHECK(is_project_closing_task IN (0, 1)),
    note TEXT NULL CHECK(note IS NULL OR length(trim(note)) > 0)
);

CREATE INDEX IF NOT EXISTS idx_tasks_date ON tasks(date);
CREATE INDEX IF NOT EXISTS idx_tasks_category_id ON tasks(category_id);
CREATE INDEX IF NOT EXISTS idx_tasks_project_id ON tasks(project_id);
CREATE INDEX IF NOT EXISTS idx_tasks_project_date ON tasks(project_id, date);
DROP INDEX IF EXISTS idx_projects_category_id;
CREATE INDEX IF NOT EXISTS idx_projects_is_done ON projects(is_done);
CREATE INDEX IF NOT EXISTS idx_projects_project_category_id ON projects(project_category_id);

DROP VIEW IF EXISTS project_with_tasks;
CREATE VIEW project_with_tasks AS
SELECT
    p.id,
    p.name,
    p.project_category_id,
    p.sum_time_length,
    p.start_date,
    p.end_date,
    p.is_done,
    COALESCE(
        (
            SELECT GROUP_CONCAT(ordered_tasks.id, ',')
            FROM (
                SELECT t.id
                FROM tasks t
                WHERE t.project_id = p.id
                ORDER BY t.date, t.id
            ) AS ordered_tasks
        ),
        ''
    ) AS tasks
FROM projects p;

DROP TRIGGER IF EXISTS tasks_after_insert_sync_project;
CREATE TRIGGER tasks_after_insert_sync_project
AFTER INSERT ON tasks
WHEN NEW.project_id IS NOT NULL
BEGIN
    UPDATE projects
    SET
        sum_time_length = COALESCE(
            (SELECT SUM(time_length) FROM tasks WHERE project_id = NEW.project_id),
            0
        ),
        start_date = (SELECT MIN(date) FROM tasks WHERE project_id = NEW.project_id),
        end_date = (
            SELECT MAX(date)
            FROM tasks
            WHERE project_id = NEW.project_id
              AND is_project_closing_task = 1
        ),
        is_done = CASE
            WHEN EXISTS(
                SELECT 1
                FROM tasks
                WHERE project_id = NEW.project_id
                  AND is_project_closing_task = 1
            ) THEN 1
            ELSE 0
        END
    WHERE id = NEW.project_id;
END;

DROP TRIGGER IF EXISTS tasks_after_update_sync_project;
CREATE TRIGGER tasks_after_update_sync_project
AFTER UPDATE OF project_id, time_length, date, is_project_closing_task ON tasks
WHEN OLD.project_id IS NOT NULL OR NEW.project_id IS NOT NULL
BEGIN
    UPDATE projects
    SET
        sum_time_length = COALESCE(
            (SELECT SUM(time_length) FROM tasks WHERE project_id = OLD.project_id),
            0
        ),
        start_date = (SELECT MIN(date) FROM tasks WHERE project_id = OLD.project_id),
        end_date = (
            SELECT MAX(date)
            FROM tasks
            WHERE project_id = OLD.project_id
              AND is_project_closing_task = 1
        ),
        is_done = CASE
            WHEN EXISTS(
                SELECT 1
                FROM tasks
                WHERE project_id = OLD.project_id
                  AND is_project_closing_task = 1
            ) THEN 1
            ELSE 0
        END
    WHERE id = OLD.project_id;

    UPDATE projects
    SET
        sum_time_length = COALESCE(
            (SELECT SUM(time_length) FROM tasks WHERE project_id = NEW.project_id),
            0
        ),
        start_date = (SELECT MIN(date) FROM tasks WHERE project_id = NEW.project_id),
        end_date = (
            SELECT MAX(date)
            FROM tasks
            WHERE project_id = NEW.project_id
              AND is_project_closing_task = 1
        ),
        is_done = CASE
            WHEN EXISTS(
                SELECT 1
                FROM tasks
                WHERE project_id = NEW.project_id
                  AND is_project_closing_task = 1
            ) THEN 1
            ELSE 0
        END
    WHERE id = NEW.project_id;
END;

DROP TRIGGER IF EXISTS tasks_after_delete_sync_project;
CREATE TRIGGER tasks_after_delete_sync_project
AFTER DELETE ON tasks
WHEN OLD.project_id IS NOT NULL
BEGIN
    UPDATE projects
    SET
        sum_time_length = COALESCE(
            (SELECT SUM(time_length) FROM tasks WHERE project_id = OLD.project_id),
            0
        ),
        start_date = (SELECT MIN(date) FROM tasks WHERE project_id = OLD.project_id),
        end_date = (
            SELECT MAX(date)
            FROM tasks
            WHERE project_id = OLD.project_id
              AND is_project_closing_task = 1
        ),
        is_done = CASE
            WHEN EXISTS(
                SELECT 1
                FROM tasks
                WHERE project_id = OLD.project_id
                  AND is_project_closing_task = 1
            ) THEN 1
            ELSE 0
        END
    WHERE id = OLD.project_id;
END;
