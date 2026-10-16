CREATE TABLE service_notes (id INTEGER PRIMARY KEY, body TEXT NOT NULL);
INSERT INTO service_notes(id, body) VALUES (1, 'ready');
CREATE TABLE service_sessions (digest BLOB PRIMARY KEY, expires BIGINT NOT NULL);
CREATE INDEX service_sessions_expiry ON service_sessions(expires);
