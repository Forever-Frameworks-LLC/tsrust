declare function require(name: string): any;
const { DatabaseSync } = require('node:sqlite');
function batch(db: any): number {
  db.exec('BEGIN IMMEDIATE');
  try {
    db.prepare('DELETE FROM items').run();
    const insert = db.prepare('INSERT INTO items VALUES (?1, ?2)');
    for (let index = 0; index < 128; index++) insert.run({ '1': index, '2': index * 2 });
    db.exec('COMMIT');
  } catch (error) { db.exec('ROLLBACK'); throw error; }
  return db.prepare('SELECT sum(value) AS total FROM items').get().total;
}
const db = new DatabaseSync(':memory:');
db.exec('CREATE TABLE items(id INTEGER PRIMARY KEY, value INTEGER NOT NULL)');
for (let warmup = 0; warmup < 8; warmup++) batch(db);
for (let sample = 0; sample < 31; sample++) {
  const start = performance.now();
  const checksum = batch(db);
  const elapsed = performance.now() - start;
  console.log(checksum); console.log(elapsed);
}
db.close();
