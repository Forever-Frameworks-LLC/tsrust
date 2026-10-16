declare function require(name: string): any;
declare const process: { env: Record<string, string | undefined> };
const { DatabaseSync } = require('node:sqlite');
const fs = require('node:fs');
const { createHash } = require('node:crypto');
const directory = 'examples/sql-migrations/migrations';
const db = new DatabaseSync(process.env.DATABASE_PATH);
db.exec('CREATE TABLE _sqlx_migrations(version BIGINT PRIMARY KEY, checksum BLOB, success BOOLEAN)');
function migrate(): void {
  const files: string[] = fs.readdirSync(directory).filter((file: string) => file.endsWith('.sql')).sort();
  const applied: any[] = db.prepare('SELECT version, checksum, success FROM _sqlx_migrations').all();
  const versions = files.map(file => Number(file.split('_')[0]));
  if (applied.some(row => !row.success || !versions.includes(row.version))) throw Error('History mismatch');
  for (const file of files) {
    const version = Number(file.split('_')[0]);
    const sql: string = fs.readFileSync(directory + '/' + file, 'utf8');
    const checksum = createHash('sha384').update(sql).digest();
    const prior = applied.find(row => row.version === version);
    if (prior) { if (!checksum.equals(prior.checksum)) throw Error('Checksum mismatch'); }
    else {
      db.exec('BEGIN');
      try { db.exec(sql); db.prepare('INSERT INTO _sqlx_migrations VALUES (?, ?, true)').run(version, checksum); db.exec('COMMIT'); }
      catch (error) { db.exec('ROLLBACK'); throw error; }
    }
  }
}
function batch(): number {
  for (let index = 0; index < 32; index++) migrate();
  return db.prepare('SELECT id FROM service_notes').get().id;
}
migrate();
for (let warmup = 0; warmup < 8; warmup++) batch();
for (let sample = 0; sample < 31; sample++) {
  const start = performance.now(); const checksum = batch();
  console.log(checksum); console.log(performance.now() - start);
}
db.close();
