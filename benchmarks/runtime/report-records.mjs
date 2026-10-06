// Optional SQLite compatibility. Original bytes and filename identities remain authoritative.
import fs from 'node:fs';
import path from 'node:path';
import {createHash} from 'node:crypto';
import {spawnSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';
import {constants as bufferConstants} from 'node:buffer';

const bridge = fileURLToPath(new URL('../storage/store.py', import.meta.url));
const hash = bytes => createHash('sha256').update(bytes).digest('hex');
function archiveCommand(args, maxBuffer = 16 * 1024 * 1024) {
 const result = spawnSync('python3', ['-B', bridge, ...args], {maxBuffer});
 if(result.error)throw result.error;
 if(result.status !== 0)throw new Error(`Benchmark archive reader failed: ${result.stderr?.toString().trim() || 'unknown error'}`);
 return result.stdout;
}
function parsedRecord(file, bytes) {
 const data = JSON.parse(bytes.toString('utf8'));
 if(!data.timestamp || data.schemaVersion !== 1)throw new Error(`Invalid run record: ${path.basename(file)}`);
 return {file, data, sha256:hash(bytes)};
}

export function loadReportRecords({recordsDirectory, archive, archivePrefix = 'runtime/results'}) {
 const records = new Map();
 let filenames;
 try { filenames = fs.readdirSync(recordsDirectory); }
 catch(error) { if(!archive || error.code !== 'ENOENT')throw error; filenames = []; }
 for(const filename of filenames.filter(name => name.endsWith('.json'))) {
  const file = path.join(recordsDirectory, filename);
  records.set(filename, parsedRecord(file, fs.readFileSync(file)));
 }
 if(archive) {
  const entries = archiveCommand(['list', archive, '--prefix', archivePrefix]).toString('utf8').trim();
  for(const line of entries ? entries.split('\n') : []) {
   const metadata = JSON.parse(line);
   const filename = metadata.path.slice(archivePrefix.length + 1);
   // Match existing discovery: only direct .json children of the results folder.
   if(!metadata.path.startsWith(archivePrefix + '/') || filename.includes('/') || !filename.endsWith('.json'))continue;
   if(!Number.isSafeInteger(metadata.original_size) || metadata.original_size < 0 || metadata.original_size > bufferConstants.MAX_LENGTH - 1024 || !/^[a-f0-9]{64}$/.test(metadata.sha256))throw new Error('Invalid archived record identity');
   const bytes = archiveCommand(['read', archive, metadata.path], Math.max(64 * 1024, metadata.original_size + 1024));
   if(bytes.length !== metadata.original_size || hash(bytes) !== metadata.sha256)throw new Error(`Archived record identity mismatch: ${metadata.path}`);
   const loose = records.get(filename);
   if(loose && loose.sha256 !== metadata.sha256)throw new Error(`Loose/archive record conflict: ${filename}`);
   if(!loose)records.set(filename, parsedRecord(path.join(recordsDirectory, filename), bytes));
  }
 }
 const values = archive ? [...records.entries()].sort(([a], [b]) => a < b ? -1 : a > b ? 1 : 0).map(([,record]) => record) : [...records.values()];
 return values.map(({file, data}) => ({file, data}));
}
