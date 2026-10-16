declare function require(name:string):any;
const { DatabaseSync } = require('node:sqlite');
const db = new DatabaseSync(':memory:');
try {
  db.exec('CREATE TABLE jobs(id TEXT PRIMARY KEY, body TEXT NOT NULL, state TEXT NOT NULL, available BIGINT NOT NULL, lease_until BIGINT NOT NULL, attempts BIGINT NOT NULL, generation BIGINT NOT NULL)');
  db.exec('CREATE TABLE effects(job_id TEXT PRIMARY KEY, body TEXT NOT NULL)');
  const batch = ():number => {
    for (let id = 0; id < 128; id++) {
      db.prepare("INSERT INTO jobs VALUES (?, ?, 'pending', ?, 0, 0, 0) ON CONFLICT(id) DO NOTHING").run(`${id}`, 'body', 0);
      if (db.prepare('SELECT body FROM jobs WHERE id = ?').get(`${id}`).body !== 'body') throw new Error('Conflicting job');
      db.prepare("UPDATE jobs SET state = 'failed' WHERE state = 'running' AND lease_until <= ? AND attempts >= 3").run(0);
      const job = db.prepare("UPDATE jobs SET state = 'running', attempts = attempts + 1, generation = generation + 1, lease_until = ? WHERE id = (SELECT id FROM jobs WHERE attempts < 3 AND ((state = 'pending' AND available <= ?) OR (state = 'running' AND lease_until <= ?)) ORDER BY available, id LIMIT 1) RETURNING id, body, attempts, generation").get(30000, 0, 0);
      if (!job) throw new Error('Missing job');
      db.exec('BEGIN');
      try {
        const changed = db.prepare("UPDATE jobs SET state = 'done', lease_until = 0 WHERE id = ? AND generation = ? AND state = 'running' AND lease_until > ?").run(job.id, job.generation, 1).changes;
        if (changed !== 1) throw new Error('Lost lease');
        db.prepare('INSERT INTO effects VALUES (?, ?) ON CONFLICT(job_id) DO NOTHING').run(job.id, job.body);
        db.exec('COMMIT');
      } catch (error) { db.exec('ROLLBACK'); throw error; }
    }
    return db.prepare('SELECT COUNT(*) AS total FROM effects').get().total;
  };
  for (let warmup = 0; warmup < 8; warmup++) { db.exec('DELETE FROM jobs'); db.exec('DELETE FROM effects'); batch(); }
  for (let sample = 0; sample < 31; sample++) {
    db.exec('DELETE FROM jobs'); db.exec('DELETE FROM effects');
    const start = performance.now(), checksum = batch();
    console.log(checksum); console.log(performance.now() - start);
  }
} finally { db.close(); }
