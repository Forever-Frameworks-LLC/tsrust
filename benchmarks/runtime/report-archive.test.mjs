import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';
import {updateReport} from './report.mjs';
import {loadReportRecords} from './report-records.mjs';

const utility = fileURLToPath(new URL('../storage/store.py', import.meta.url));
function python(...args) {
 const result = spawnSync('python3', ['-B', ...args], {encoding:'utf8'});
 assert.equal(result.status, 0, result.stderr);
 return result.stdout;
}
function fixture(timestamp = '2026-01-01T00:00:00Z') {
 return {
  schemaVersion:1, timestamp, verifiedOnly:false, semanticEpoch:'rust-native-utf8-v1', measurementScope:'execution-only',
  manifest:{gate:31, processes:3, samples:31, warmups:8, workloads:[{id:'scalar_control', iterations:1}]},
  machine:{cpu:'Test', architecture:'arm64', logicalCPUs:1, totalMemoryBytes:2**30, platform:'test', release:'test'},
  versions:{node:'test', bun:'test', deno:'test', typescript:'test', tsx:'test', 'ts-node':'test', rust:'test'},
  correctness:{casesPerWorkload:5, checksums:{scalar_control:7}}, sources:[{id:'scalar_control'}],
  execution:['native','node-tsc'].flatMap(engine => Array.from({length:3}, (_,processIndex) => ({workload:'scalar_control', engine, processIndex, checksum:7, durations:Array(31).fill(1)}))),
  compilation:[], startup:[], originalLabel:'é😀'
 };
}
function setup(t) {
 const root = fs.mkdtempSync(path.join(os.tmpdir(), 'tsrust-archive-report-'));
 t.after(() => fs.rmSync(root, {recursive:true}));
 const source = path.join(root,'source'), results = path.join(source,'runtime/results'), database = path.join(root,'records.sqlite');
 fs.mkdirSync(results, {recursive:true});
 return {root, source, results, database};
}
function write(directory, filename, data) {
 const raw = JSON.stringify(data,null,2) + '\n\n';
 fs.writeFileSync(path.join(directory,filename),raw);
 return raw;
}
function reportOptions(directory, root, name) {
 return {recordsDirectory:directory, output:path.join(root,name)};
}

test('exact tree restoration and archive-only reading reproduce existing report bytes', t => {
 const {root,source,results,database} = setup(t);
 const raw = write(results,'original.json',fixture());
 const expected = updateReport(reportOptions(results,root,'loose.md'));
 python(utility,'import',database,'runtime/results','--root',source);
 const restored = path.join(root,'restored'); fs.mkdirSync(restored);
 python(utility,'restore',database,'--root',restored);
 const restoredResults = path.join(restored,'runtime/results');
 assert.equal(fs.readFileSync(path.join(restoredResults,'original.json'),'utf8'),raw);
 assert.equal(updateReport(reportOptions(restoredResults,root,'restored.md')),expected);
 const absent = path.join(root,'archive-only-results');
 assert.equal(updateReport({...reportOptions(absent,root,'archive.md'),archive:database}),expected);
 assert.match(expected,/\[Raw JSON\]\(\.\/benchmarks\/runtime\/results\/original\.json\)/);
 assert.equal(fs.readFileSync(path.join(results,'original.json'),'utf8'),raw);
});

test('mixed records deduplicate identical bindings and preserve original filenames', t => {
 const {root,source,results,database} = setup(t);
 write(results,'archived.json',fixture());
 write(results,'loose.json',fixture('2026-01-02T00:00:00Z'));
 const expected = updateReport(reportOptions(results,root,'original.md'));
 python(utility,'import',database,'runtime/results','--root',source);
 fs.unlinkSync(path.join(results,'archived.json')); // Disposable fixture only.
 const records = loadReportRecords({recordsDirectory:results,archive:database});
 assert.deepEqual(records.map(row=>path.basename(row.file)),['archived.json','loose.json']);
 assert.equal(updateReport({...reportOptions(results,root,'mixed.md'),archive:database}),expected);
 assert.equal(records[0].data.originalLabel,'é😀');
});

test('loose/archive byte conflict fails before replacing the valid report', t => {
 const {root,source,results,database} = setup(t);
 const raw = write(results,'original.json',fixture());
 const options = reportOptions(results,root,'report.md');
 const expected = updateReport(options);
 python(utility,'import',database,'runtime/results','--root',source);
 fs.writeFileSync(path.join(results,'original.json'),raw+' '); // Same parsed object, changed bytes.
 assert.throws(()=>updateReport({...options,archive:database}),/Loose\/archive record conflict/);
 assert.equal(fs.readFileSync(options.output,'utf8'),expected);
});

test('corrupt archived bytes fail before replacing the valid report', t => {
 const {root,source,results,database} = setup(t);
 write(results,'original.json',fixture());
 const options = reportOptions(results,root,'report.md');
 const expected = updateReport(options);
 python(utility,'import',database,'runtime/results','--root',source);
 python('-c',"import sqlite3,sys,zlib; db=sqlite3.connect(sys.argv[1]); db.execute('DROP TRIGGER immutable_blobs_update'); db.execute('UPDATE blobs SET compressed=?',(zlib.compress(b\"corrupt\"),)); db.commit()",database);
 assert.throws(()=>updateReport({...options,archive:database}),/corrupt blob/);
 assert.equal(fs.readFileSync(options.output,'utf8'),expected);
});

test('archive discovery uses the configured results prefix and ignores nested JSON', t => {
 const {root,source,results,database} = setup(t);
 fs.mkdirSync(path.join(results,'nested'));
 fs.writeFileSync(path.join(results,'nested','ignore.json'),'not a run record');
 write(results,'original.json',fixture());
 python(utility,'import',database,'runtime/results','--root',source);
 assert.equal(loadReportRecords({recordsDirectory:path.join(root,'absent'),archive:database}).length,1);
 const alternate = path.join(root,'alternate'); fs.mkdirSync(alternate);
 const custom = path.join(alternate,'custom'); fs.mkdirSync(custom);
 write(custom,'original.json',fixture());
 const alternateDatabase = path.join(root,'alternate.sqlite');
 python(utility,'import',alternateDatabase,'custom','--root',alternate);
 // Another directory is not a child of the default results prefix.
 assert.equal(loadReportRecords({recordsDirectory:path.join(root,'absent'),archive:alternateDatabase}).length,0);
 assert.equal(loadReportRecords({recordsDirectory:path.join(root,'absent'),archive:alternateDatabase,archivePrefix:'custom'}).length,1);
 assert.throws(()=>loadReportRecords({recordsDirectory:path.join(root,'absent'),archive:database,archivePrefix:'../escape'}),/without traversal/);
});
