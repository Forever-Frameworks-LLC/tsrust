// Generates the public performance page from benchmarks/measurements.sqlite.
// The runs to publish are listed in performance-runs.json by run timestamp, so
// the page changes only when that list or those measurements change.
import fs from 'node:fs';
import path from 'node:path';
import { createHash } from 'node:crypto';
import { fileURLToPath } from 'node:url';
import { DatabaseSync } from 'node:sqlite';

const here = path.dirname(fileURLToPath(import.meta.url));
const root = path.resolve(here, '../..');
const config = JSON.parse(fs.readFileSync(path.join(here, 'performance-runs.json'), 'utf8'));
const manifest = JSON.parse(fs.readFileSync(path.join(root, 'benchmarks/runtime/manifest.json'), 'utf8'));
const workloads = new Map(manifest.workloads.map((workload) => [workload.id, workload]));
const output = path.join(root, 'website/src/content/docs/project/performance.md');

// One row per workload and engine; `history` lists runs as {at, ms, ops, samples}.
const db = new DatabaseSync(path.join(root, 'benchmarks/measurements.sqlite'), { readOnly: true });
const runsOf = (engine) => new Map(db.prepare('SELECT workload, history FROM measurements WHERE engine = ?').all(engine)
  .map(({ workload, history }) => [workload, new Map(JSON.parse(history).map((entry) => [entry.at, entry]))]));
const nativeRuns = runsOf('native');
const nodeRuns = runsOf('node-tsc');
const rustRuns = runsOf('rust-hand');
db.close();
const selected = new Map();
const runs = [];
const selectedNode = new Map();
for (const at of [...config.runs].sort()) {
  for (const [workload, history] of nodeRuns) {
    const measurement = history.get(at);
    if (measurement) selectedNode.set(workload, { ...measurement, at });
  }
}
for (const at of [...config.runs].sort()) {
  let matched = 0;
  for (const [workload, history] of [...nativeRuns].sort(([a], [b]) => a.localeCompare(b))) {
    const native = history.get(at);
    const node = selectedNode.get(workload);
    if (native && node) {
      const expected = config.processes * config.samplesPerProcess;
      if (native.samples !== expected || node.samples !== expected) throw Error(`Unexpected sample count for ${workload} at ${at}`);
      selected.set(workload, { workload, at, nodeAt: node.at, native: native.ops, native_samples: native.samples, node: node.ops, node_samples: node.samples });
      matched++;
    }
  }
  if (!matched) throw Error(`Run ${at} is not in benchmarks/measurements.sqlite`);
  runs.push({ timestamp: at });
}
const rows = [...selected.values()];
for (const row of rows) {
  const baseline = config.rustBaselines?.[row.workload];
  if (!baseline) continue;
  const source = fs.readFileSync(path.join(root, 'benchmarks/runtime/parity', `${row.workload}.rs`));
  if (createHash('sha256').update(source).digest('hex') !== baseline.sourceSha256) throw Error(`Stale Rust baseline source: ${row.workload}`);
  const measurement = rustRuns.get(row.workload)?.get(baseline.at);
  if (!measurement || measurement.samples !== config.processes * config.samplesPerProcess) throw Error(`Missing Rust baseline: ${row.workload}`);
  row.rust = measurement.ops;
  row.rustAt = baseline.at;
}
if (!rows.length) throw Error('No native/node-tsc comparisons in the configured runs');

// Manifest categories grew per feature family; fold them into reader-facing groups.
const families = [
  ['Strings', /^(strings)$/, /(^|_)(string|ascii|unicode|text|template|substr|char)/],
  ['Arrays', /^(arrays|array operations)$/, /(^|_)array|arrays$/],
  ['Numbers and operators', /^(numbers|operators|booleans)$/, /^(number|math|int|boolean|compound|update|scalar)_/],
  ['Functions and callbacks', /^(callbacks|closures and callbacks|recursion)$/, /(closure|callback|callable|factory|capture|function|higher)/],
  ['JSON and schema', /^(json|schema)$/, /(json|schema)/],
  ['Standard library', /^(bytes|resources|paths|os|environment|time|collections|module-state|testing)$/, /^(native|std)_/],
  ['Records and objects', /^(records|other)$/, /(record|object|clone|destructur)/],
];
const groupOf = (id, workload) => {
  const category = String(workload?.category ?? '').toLowerCase();
  for (const [name, categories, ids] of families) if (category && categories.test(category)) return name;
  for (const [name, , ids] of families) if (ids.test(id)) return name;
  return 'Core language';
};
const groups = new Map();
for (const row of rows) {
  const workload = workloads.get(row.workload);
  const group = config.groups[row.workload] ?? groupOf(row.workload, workload);
  if (!groups.has(group)) groups.set(group, []);
  groups.get(group).push({ ...row, description: workload?.description });
}

const rate = (value) => {
  if (value >= 1e9) return `${(value / 1e9).toFixed(2)} G`;
  if (value >= 1e6) return `${(value / 1e6).toFixed(2)} M`;
  if (value >= 1e3) return `${(value / 1e3).toFixed(1)} k`;
  return value.toFixed(0);
};
const ratio = (row) => row.native / row.node;
const cell = (text) => String(text).replaceAll('|', '\\|').replaceAll('<', '&lt;');
const ratios = rows.map(ratio).sort((a, b) => a - b);
const median = ratios.length % 2 ? ratios[(ratios.length - 1) / 2] : (ratios[ratios.length / 2 - 1] + ratios[ratios.length / 2]) / 2;
const faster = ratios.filter((value) => value > 1).length;
const machine = config.machine ?? {};
const samples = rows[0].native_samples;
const httpManifest=JSON.parse(fs.readFileSync(path.join(root,'benchmarks/http/requests-manifest.json'),'utf8'));
if(config.httpBaselines&&config.httpProtocolSha256!==createHash('sha256').update(JSON.stringify({semanticEpoch:httpManifest.semanticEpoch,protocol:httpManifest.protocol,workloads:httpManifest.workloads})).digest('hex'))throw Error('Stale HTTP measurement protocol');
const httpRows=[];
for(const workload of httpManifest.workloads){
  const pins=config.httpBaselines?.[workload.id];if(!pins)continue;
  const engines={};
  for(const [engine,history,file] of [['native',nativeRuns,'requests.tsr'],['node-tsc',nodeRuns,'requests.ts'],['rust-hand',rustRuns,'requests.rs']]){
    const pin=pins[engine],source=fs.readFileSync(path.join(root,'benchmarks/http',file));
    if(!pin||createHash('sha256').update(source).digest('hex')!==pin.sourceSha256)throw Error(`Stale HTTP source: ${workload.id}/${engine}`);
    const measured=history.get(workload.id)?.get(pin.at);
    if(!measured||measured.samples!==httpManifest.protocol.processes*httpManifest.protocol.samples)throw Error(`Missing HTTP measurement: ${workload.id}/${engine}`);
    engines[engine]={...measured,at:pin.at};
  }
  httpRows.push({...workload,engines});
}
let httpSection='## HTTP requests\n\n';
if(!httpRows.length){httpSection+='HTTP request workloads are registered; measured results are not published yet.\n\n';}
else{
  httpSection+=`These end-to-end loopback tests use the same Node HTTP client for every server. Each sample completes and validates 128 responses, using either one or 16 keep-alive connections. Payloads are prepared before timing; server startup, builds and database seeding are excluded. Rates include client I/O and complete response validation, so they are not isolated server saturation limits. Rates average requests per second over all samples; batch times are median elapsed milliseconds for 128 requests.\n\n`;
  httpSection+='| Workload | Connections | Tsrust (req/s) | Node.js (req/s) | Hand-written Rust (req/s) | Tsrust / Node.js | Median batch ms: Tsrust / Node / Rust | Dates: Tsrust / Node / Rust |\n| --- | ---: | ---: | ---: | ---: | ---: | ---: | --- |\n';
  for(const row of httpRows){const n=row.engines.native,j=row.engines['node-tsc'],r=row.engines['rust-hand'];httpSection+=`| ${row.name} | ${row.concurrency} | ${rate(n.ops)} | ${rate(j.ops)} | ${rate(r.ops)} | ${(n.ops/j.ops).toFixed(2)}× | ${n.ms.toFixed(3)} / ${j.ms.toFixed(3)} / ${r.ms.toFixed(3)} | ${n.at.slice(0,10)} / ${j.at.slice(0,10)} / ${r.at.slice(0,10)} |\n`;}
  httpSection+='\nText responses contain UTF-8 text; JSON responses serialize a record per request. The parse workload validates a posted record and transforms its fields. The database workload parses an ID, performs a parameterized SQLite lookup and serializes the returned row. IDs vary across 128 seeded rows.\n\n';
  httpSection+='SQLite is in memory and uses one connection. Tsrust and Rust use an asynchronous SQLx pool; Node uses its built-in synchronous prepared SQLite statement. These are warm read queries, with no disk or write-throughput claim.\n\n';
  if(config.httpSqliteVersions)httpSection+=`SQLite versions: Tsrust ${cell(config.httpSqliteVersions.native)}, Node.js ${cell(config.httpSqliteVersions['node-tsc'])}, Rust ${cell(config.httpSqliteVersions['rust-hand'])}.\n\n`;
  const dates=[...new Set(httpRows.flatMap(row=>Object.values(row.engines).map(engine=>engine.at.slice(0,10))))];
  httpSection+=`${httpManifest.protocol.processes*httpManifest.protocol.samples} samples per server/workload, after ${httpManifest.protocol.warmups} warmups in each process. Measured ${dates.join(', ')} on the machine above. Request status, content type and every body byte are checked; malformed input and missing-row cases are verified before timing. Sources and commands: \`benchmarks/http/README.md\`.\n\n`;
}


let page = `---
title: "Performance"
description: "Measured Tsrust, Node.js and hand-written Rust execution speed."
---

The tables compare three implementations of the same checked workloads:

- **Tsrust**: Tsrust source compiled to Rust, then built as an executable by the Rust compiler.
- **Node.js**: equivalent TypeScript compiled by \`tsc\` and run on Node.js.
- **Hand-written Rust**: independently written Rust using ordinary Rust ownership,
  standard-library operations and established crates where needed.

Higher operations per second is faster. A **Tsrust / Node.js** ratio above 1
means Tsrust was faster; a **Tsrust / Rust** ratio above 1 means Tsrust was
faster than the hand-written Rust implementation.

This snapshot uses the latest accepted measurement selected for each workload.
Each engine has its own measurement date because reference baselines are reused.
Rejected experiments and candidates still in validation are excluded. A dash in
the Rust columns means that comparison has not been measured. The Rust win count
covers only measured comparisons; unmeasured workloads are not losses.

## Summary

This summary covers the language/library workloads. HTTP requests use a separate end-to-end protocol and table below.

| Measure | Value |
| --- | --- |
| Workloads compared | ${rows.length} |
| Hand-written Rust faster than Node.js | ${rows.filter(row => row.rust !== undefined && row.rust > row.node).length} of ${rows.length} (${rows.filter(row => row.rust !== undefined).length} measured) |
| Tsrust faster than Node.js | ${faster} of ${rows.length} |
| Median ratio (Tsrust ÷ Node.js) | ${median.toFixed(2)}× |
| Ratio range (Tsrust ÷ Node.js) | ${ratios[0].toFixed(2)}× to ${ratios.at(-1).toFixed(2)}× |
| Machine | ${cell(machine.cpu ?? 'unrecorded')} (${cell(machine.platform ?? '')} ${cell(machine.architecture ?? '')}) |
| Toolchains | Rust ${cell(config.rust)}; Node.js ${cell(config.node)}; TypeScript ${cell(config.typescript)} |
| Samples | ${samples} timed samples per engine and workload |
| Measured | ${runs.map((run) => run.timestamp.slice(0, 10)).filter((value, index, all) => all.indexOf(value) === index).join(', ')} |

${httpSection}## Results by group

`;
const middle = (values) => { const sorted = [...values].sort((a, b) => a - b); const half = sorted.length >> 1; return sorted.length % 2 ? sorted[half] : (sorted[half - 1] + sorted[half]) / 2; };
const ordered = [...groups].sort(([a], [b]) => a.localeCompare(b));
page += '| Group | Workloads | Tsrust faster than Node.js | Median ratio |\n| --- | ---: | ---: | ---: |\n';
for (const [group, entries] of ordered) {
  const values = entries.map(ratio);
  page += `| [${group}](#${group.toLowerCase().replaceAll(/[^a-z0-9]+/g, '-')}) | ${entries.length} | ${values.filter((value) => value > 1).length} | ${middle(values).toFixed(2)}× |\n`;
}
const slower = ordered.filter(([, entries]) => middle(entries.map(ratio)) < 1).map(([group]) => `the ${group} group`);
page += slower.length
  ? `\nTsrust is not uniformly faster than Node.js. In ${slower.join(' and ')}, the median workload runs slower than on Node.js: these remain optimization targets. A throughput table alone does not identify the cause of a performance gap.\n\n`
  : '\n';
page += 'Each group lists every workload, slowest ratio first. Speeds are operations per second, abbreviated: k is thousands, M is millions and G is billions, so `5.35 M` is about 5,350,000 operations per second. The ratio column is the one to compare.\n\n';
for (const [group, entries] of ordered) {
  page += `### ${group}\n\n<details>\n<summary>${entries.length} workloads</summary>\n\n| Workload | Tsrust (ops/s) | Node.js (ops/s) | Tsrust / Node.js | Hand-written Rust (ops/s) | Tsrust / Rust | Tsrust date | Node date | Rust date |\n| --- | ---: | ---: | ---: | ---: | ---: | --- | --- | --- |\n`;
  for (const row of entries.sort((a, b) => ratio(a) - ratio(b))) {
    const label = row.description ? `\`${row.workload}\`: ${cell(row.description)}` : `\`${row.workload}\``;
    page += `| ${label} | ${rate(row.native)} | ${rate(row.node)} | ${ratio(row).toFixed(2)}× | ${row.rust === undefined ? "—" : rate(row.rust)} | ${row.rust === undefined ? "—" : (row.native / row.rust).toFixed(2) + "×"} | ${row.at.slice(0,10)} | ${row.nodeAt.slice(0,10)} | ${row.rustAt?.slice(0,10) ?? "—"} |\n`;
  }
  page += '\n</details>\n\n';
}
page += `Rates are mean operations per second over every timed sample. Each workload
row is one complete workload: input setup, the operations and result retention
are timed together, so a rate is not the cost of one isolated operation.

## Method

- **Tsrust**: the Tsrust compiler emits Rust, built with the pinned Rust
  toolchain at optimization level 3. That is the Rust compiler's highest speed
  setting, the same one \`tsrust build --release\` uses. Integer overflow checks
  stay enabled, so the measured programs keep their safety checks.
- **Node.js**: the matching TypeScript program is compiled once with \`tsc\` and
  run with the pinned Node.js version. JIT warm-up runs before timing.
- Each engine runs ${config.processes} separate processes with ${config.warmups} warm-up
  batches and ${config.samplesPerProcess} timed samples per process. Startup and
  compilation time are excluded.
- Every correctness case is checked against an independent oracle before timing,
  and the timed result is verified afterwards.
- Tsrust strings use UTF-8 semantics. Where a workload depends on string encoding,
  the TypeScript counterpart performs the same UTF-8 work explicitly.

## Reusable reference baselines

Hand-written Rust uses ordinary Rust ownership and standard-library operations,
with established crates where the workload requires them (for example,
serde_json for JSON). It must preserve the same observable results, errors and
safety checks. It is not generated Tsrust output with a different label.

Node.js and hand-written Rust measurements are reusable reference baselines;
they need not run for every compiler optimization. Refresh them when workload
semantics, benchmark protocol, reference source, toolchains or hardware change.
A fresh control may also be needed to resolve noisy or close timing results.
Dates remain visible because baselines and compiler measurements may be from
different runs. Missing Rust results are not inferred from Node.js timings.

## Limits

These are workload-level comparisons on one machine, not a general claim that
every Tsrust program runs faster than its Node.js counterpart. Short workloads are sensitive to ordinary
machine variation, and small ratio differences are not significant. Rows where
Node.js is faster are kept and reviewed rather than removed.

The measurements are in \`benchmarks/measurements.sqlite\` in the repository:
one row per workload and engine, with each run's date, median sample time,
mean speed and sample count. It can be queried with the standard SQLite tools.
The benchmark runner and every workload source pair are in
\`benchmarks/runtime\`; HTTP server comparisons are in \`benchmarks/http\`.
`;
fs.mkdirSync(path.dirname(output), { recursive: true });
fs.writeFileSync(output, page);
console.log(`Wrote ${path.relative(root, output)} with ${rows.length} comparisons from ${runs.length} run(s).`);
