import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import crypto from 'node:crypto';
import {spawnSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';
import {rustChecksum, numberAdapterSources, adaptNumberMemoryDriver} from './adapters.mjs';
import {expectedNumber} from './number-oracle.mjs';
import {isNumberResult, validateNumberObservation} from './number-observation.mjs';
import {render} from './report.mjs';
const here = path.dirname(fileURLToPath(import.meta.url));
const root = path.resolve(here, '../..');
const build = path.join(root, 'target/runtime-comparisons/allocations');
fs.mkdirSync(build, {recursive: true});
const allProbes = JSON.parse(fs.readFileSync(path.join(here, 'allocations/manifest.json')));
const selected = process.argv.find(arg=>arg.startsWith('--only='))?.slice(7).split(',');
const probes = allProbes.filter(probe=>!selected || selected.includes(probe.id));
if (!probes.length || selected?.some(id=>!allProbes.some(probe=>probe.id===id))) throw new Error('Unknown allocation probe');
const pins = JSON.parse(fs.readFileSync(path.join(here, 'versions.json')));
if (process.version !== pins.node) throw new Error('Use the pinned Node version');
const env = {...process.env, PATH: path.dirname(process.execPath) + path.delimiter + process.env.PATH};
delete env.NODE_OPTIONS;
function run(command) {
 const child = spawnSync(command[0], command.slice(1).map(String), {cwd: here, env, encoding: 'utf8', timeout: 120000});
 if (child.error || child.status !== 0) throw new Error(`${command[0]}: ${child.error ?? (child.stderr + child.stdout)}`);
 return child.stdout.trim();
}
const rustc = ['rustup', 'run', pins.rust, 'rustc'];
const rustVersion = run([...rustc, '--version']);
if (rustVersion.split(' ')[1] !== pins.rust) throw new Error('Rust version drift');
const tsVersion = JSON.parse(fs.readFileSync(path.join(here, 'node_modules/typescript/package.json'))).version;
if (tsVersion !== pins.typescript) throw new Error('TypeScript version drift');
run(['cargo', 'build', '--release', '--offline', '--quiet', '--manifest-path', path.join(root, 'Cargo.toml')]);
const compiler = path.join(root, 'target/release/tsrust');
const driverRS = fs.readFileSync(path.join(here, 'allocation-driver.rs'), 'utf8');
const driverTS = fs.readFileSync(path.join(here, 'allocation-driver.ts'), 'utf8');
const hash = text => crypto.createHash('sha256').update(text).digest('hex');
function oracle(id, n) {
 if (id==='number_value') return expectedNumber(id,n,7).checksum;
 switch (id) {
  case 'empty_control': return 0;
  case 'int_value': return n + 7;
  case 'boolean_value': return Number(n > 7);
  case 'string_value': return 293;
  case 'record_value': return n + 7 + Number(n > 7);
  case 'int_array': return n * 7 + n * (n - 1) / 2;
  case 'boolean_array': return Math.ceil(n / 2);
  case 'string_array': return n * 293;
  default: throw new Error('Unknown probe');
 }
}
const rows = [], sources = [];
for (const probe of probes) {
 const source = fs.readFileSync(path.join(here, 'allocations', `${probe.id}.tsr`), 'utf8');
 const counterpart = fs.readFileSync(path.join(here, 'allocations', `${probe.id}.ts`), 'utf8');
 const emitted = run([compiler, 'emit', path.join(here, 'allocations', `${probe.id}.tsr`)]);
 if (!emitted.includes(`fn function0(value0: i64, value1: i64) -> ${probe.rustType}`) || !emitted.includes('fn main()')) throw new Error('Emitter allocation-probe ABI changed');
 const instrumented = emitted.replace('fn main()', 'fn source_main()') + '\n' + (isNumberResult(probe)?rustChecksum(probe,{probe:true}):probe.rustChecksum) + '\n' + (isNumberResult(probe)?adaptNumberMemoryDriver(driverRS,{native:true}):driverRS);
 fs.writeFileSync(path.join(build, `${probe.id}.rs`), instrumented);
 fs.writeFileSync(path.join(build, `${probe.id}.ts`), counterpart + '\n' + (isNumberResult(probe)?numberAdapterSources['number-observation.ts']+'\nfunction probeChecksum(value: unknown): number { return observeNumber(value as number).checksum; }\n':'') + (isNumberResult(probe)?adaptNumberMemoryDriver(driverTS):driverTS));
 fs.writeFileSync(path.join(build, `${probe.id}.json`), JSON.stringify({extends: path.join(here, 'tsconfig.json'), files: [path.join(build, `${probe.id}.ts`)]}));
 const nativeCommand = [...rustc, '--edition=2024', '-C', 'opt-level=3', '-C', 'overflow-checks=on', '-A', 'warnings', path.join(build, `${probe.id}.rs`), '-o', path.join(build, probe.id)];
 const tsCommand = [process.execPath, path.join(here, 'node_modules/typescript/bin/tsc'), '--project', path.join(build, `${probe.id}.json`)];
 run(nativeCommand); run(tsCommand);
 sources.push({id: probe.id, tsrSha256: hash(source), tsSha256: hash(counterpart), emittedSha256: hash(emitted), instrumentedSha256: hash(instrumented), nativeCompile: nativeCommand, tscCompile: tsCommand});
 for (const n of probe.sizes) {
  const samples = [];
  for (let sample = 0; sample < 3; sample++) {
   const native = JSON.parse(run([path.join(build, probe.id), n]));
   const node = JSON.parse(run([process.execPath, '--expose-gc', path.join(build, `${probe.id}.js`), n]));
   const expected = oracle(probe.id, n);
   if (isNumberResult(probe)) { const observation=expectedNumber(probe.id,n,7); validateNumberObservation(native,observation); validateNumberObservation(node,observation); }
   if (native.checksum !== expected || node.checksum !== expected || native.liveBytesAfterDrop !== 0) throw new Error(`Allocation correctness failed: ${probe.id}/${n}`);
   samples.push({native, node});
  }
  for (const key of ['allocations','reallocations','deallocationsDuringWork','requestedBytes','retainedRequestedBytes','peakRequestedBytes']) if (new Set(samples.map(s => s.native[key])).size !== 1) throw new Error(`Nondeterministic native allocation count: ${probe.id}/${key}`);
  rows.push({probe: probe.id, n, samples});
 }
 console.log(`Allocation probe validated: ${probe.id}`);
}
const data = {semanticEpoch: JSON.parse(fs.readFileSync(path.join(here, 'manifest.json'))).semanticEpoch, timestamp: new Date().toISOString(), schemaVersion: 1, machine: {cpu: os.cpus()[0]?.model, architecture: os.arch(), release: os.release()}, versions: {node: process.version, rust: rustVersion, typescript: tsVersion}, probes, sources, rows, nodeComponents: process.versions, oracleInputs: ['oracle.mjs','number-oracle.mjs','number-predicate-oracle.mjs','math-oracle.mjs','exact-math-oracle.mjs','slice-oracle.mjs','temporary-index-oracle.mjs','map-filter-oracle.mjs','fixed-number-oracle.mjs','array-predicate-oracle.mjs','string-construction-oracle.mjs'].map(name=>[name,hash(fs.readFileSync(path.join(here,name)))]), numberResultEncoding: 'ieee-f64-bits-v1', numberAdapterInputs: [...Object.entries(numberAdapterSources),...['number-observation.mjs','number-oracle.mjs'].map(name=>[name,fs.readFileSync(path.join(here,name),'utf8')])].map(([name,source])=>[name,hash(source)]), driverRSSha256: hash(driverRS), driverTSSha256: hash(driverTS), harnessSha256: hash(fs.readFileSync(fileURLToPath(import.meta.url))), compilerInputs: fs.readdirSync(path.join(root, 'crates/tsrust-compiler/src')).filter(n => n.endsWith('.rs')).sort().map(n => [n, hash(fs.readFileSync(path.join(root, 'crates/tsrust-compiler/src', n)))])};
const file = path.join(here, 'results', `${data.timestamp.replace(/[:.]/g, '-')}-allocations.json`);
fs.writeFileSync(file, JSON.stringify(data, null, 2) + '\n', {flag: 'wx'});
console.log(`Saved ${file}`);
if(process.argv.includes('--report') && !process.argv.includes('--defer-report'))render(file);
