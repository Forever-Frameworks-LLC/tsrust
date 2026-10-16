declare function require(name:string):any;
const { DatabaseSync } = require('node:sqlite');
async function main():Promise<void> {
  const db = new DatabaseSync(':memory:');
  try {
    db.exec('CREATE TABLE tsrust_cursor_bench(id BIGINT PRIMARY KEY, value BIGINT NOT NULL)');
    const insert = db.prepare('INSERT INTO tsrust_cursor_bench VALUES (?, ?)');
    db.exec('BEGIN');
    for (let index = 0; index < 128; index++) insert.run(index, index * 2);
    db.exec('COMMIT');
    const select = db.prepare('SELECT value FROM tsrust_cursor_bench ORDER BY id');
    const batch = ():number => {
      let sum = 0, count = 0;
      for (const row of select.iterate()) {
        if (row.value !== count * 2) throw new Error('Unexpected row');
        sum += row.value;
        count++;
      }
      if (count !== 128) throw new Error('Unexpected row count');
      return sum;
    };
    for (let warmup = 0; warmup < 8; warmup++) batch();
    for (let sample = 0; sample < 31; sample++) {
      const start = performance.now();
      const checksum = batch();
      console.log(checksum);
      console.log(performance.now() - start);
    }
  } finally { db.close(); }
}
void main();
