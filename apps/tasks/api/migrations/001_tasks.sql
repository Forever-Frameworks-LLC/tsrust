CREATE TABLE tasks (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL CHECK (length(trim(title)) > 0),
    description TEXT NOT NULL DEFAULT '',
    status TEXT NOT NULL DEFAULT 'todo' CHECK (status IN ('todo', 'in_progress', 'done')),
    priority INTEGER NOT NULL DEFAULT 2 CHECK (priority BETWEEN 0 AND 3),
    assignee TEXT NOT NULL DEFAULT '',
    due_date TEXT,
    version BIGINT NOT NULL DEFAULT 1,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE task_changes (
    sequence BIGSERIAL PRIMARY KEY,
    operation TEXT NOT NULL CHECK (operation IN ('upsert', 'delete')),
    task_id TEXT NOT NULL,
    payload JSONB NOT NULL
);

-- Serialize mutations before allocating change sequences so commits cannot leave
-- an unseen lower sequence behind a cursor already acknowledged by a client.
CREATE FUNCTION lock_task_changes() RETURNS TRIGGER LANGUAGE plpgsql AS $$
BEGIN
    PERFORM pg_advisory_xact_lock(785231049);
    RETURN NULL;
END;
$$;

CREATE TRIGGER tasks_change_lock
BEFORE INSERT OR UPDATE OR DELETE ON tasks
FOR EACH STATEMENT EXECUTE FUNCTION lock_task_changes();

CREATE FUNCTION version_task() RETURNS TRIGGER LANGUAGE plpgsql AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        NEW.version := 1;
    ELSE
        NEW.version := OLD.version + 1;
    END IF;
    NEW.updated_at := clock_timestamp();
    RETURN NEW;
END;
$$;

CREATE TRIGGER tasks_version
BEFORE INSERT OR UPDATE ON tasks
FOR EACH ROW EXECUTE FUNCTION version_task();

CREATE FUNCTION record_task_change() RETURNS TRIGGER LANGUAGE plpgsql AS $$
BEGIN
    IF TG_OP = 'DELETE' THEN
        INSERT INTO task_changes(operation, task_id, payload)
        VALUES ('delete', OLD.id, jsonb_build_object('id', OLD.id, 'version', OLD.version));
        RETURN OLD;
    END IF;
    INSERT INTO task_changes(operation, task_id, payload)
    VALUES ('upsert', NEW.id, to_jsonb(NEW));
    RETURN NEW;
END;
$$;

CREATE TRIGGER tasks_record_change
AFTER INSERT OR UPDATE OR DELETE ON tasks
FOR EACH ROW EXECUTE FUNCTION record_task_change();
