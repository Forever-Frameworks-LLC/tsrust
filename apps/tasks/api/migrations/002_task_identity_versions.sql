CREATE INDEX task_changes_identity_version ON task_changes(task_id, ((payload->>'version')::bigint));

-- Recreating an id must not let an old editor version match a different task.
CREATE OR REPLACE FUNCTION version_task() RETURNS TRIGGER LANGUAGE plpgsql AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        SELECT COALESCE(MAX((payload->>'version')::bigint), 0) + 1 INTO NEW.version
        FROM task_changes WHERE task_id = NEW.id;
    ELSE
        NEW.version := OLD.version + 1;
    END IF;
    NEW.updated_at := clock_timestamp();
    RETURN NEW;
END;
$$;
