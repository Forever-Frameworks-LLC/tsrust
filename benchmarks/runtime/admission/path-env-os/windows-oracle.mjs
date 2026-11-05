// New lexical admission controls only; original timed Node sources stay intact.
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

assert.equal(process.version, 'v24.11.1');
const script = fileURLToPath(import.meta.url);
if (process.argv[2] === '--child') {
  const row = JSON.parse(process.argv[3]);
  let cwdReads = 0;
  let driveReads = 0;
  process.cwd = () => {
    cwdReads++;
    if (row[0].startsWith('@error:')) throw new Error(row[0]);
    return row[0];
  };
  // Windows environment names are case insensitive. These are isolated injected
  // drive contexts, not reads or writes of the parent environment.
  const drive = row[1].startsWith('@error:') ? 'C:' : row[1].slice(0, 2);
  const drives = row[1] ? { [`=${drive.toUpperCase()}`]: row[1], [`=${drive.toLowerCase()}`]: row[1] } : {};
  process.env = new Proxy(drives, {
    get(target, key) {
      if (typeof key === 'string' && key.startsWith('=')) driveReads++;
      if (typeof target[key] === 'string' && target[key].startsWith('@error:')) throw new Error(target[key]);
      return target[key];
    },
  });
  let actual;
  try {
    actual = path.win32.resolve(...row.slice(5));
  } catch (error) {
    actual = error.message;
  }
  process.stdout.write(JSON.stringify({ actual, cwdReads, driveReads }));
  process.exit(0);
}

const rows = readFileSync(new URL('./windows-resolve.tsv', import.meta.url), 'utf8')
  .split('\n').filter((line) => line && !line.startsWith('#')).map((line) => line.split('\t'));
for (const row of rows) {
  const child = spawnSync(process.execPath, [script, '--child', JSON.stringify(row)], {
    encoding: 'utf8', env: {}, timeout: 10_000,
  });
  assert.equal(child.error, undefined);
  assert.equal(child.status, 0, child.stderr);
  assert.deepEqual(JSON.parse(child.stdout), {
    actual: row[2], cwdReads: Number(row[3]), driveReads: Number(row[4]),
  }, JSON.stringify(row));
}
console.log(`B03 Windows lexical oracle checked ${rows.length} cases`);
