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
let httpSection='### I/O and request overhead\n\n';
httpSection+='These small responses mostly measure transport, scheduling and serialization. They are useful context for a backend, but do not demonstrate a CPU-compute advantage. All results remain visible.\n\n<details>\n<summary>Text, JSON and SQLite comparisons</summary>\n\n';
for(const concurrency of [1,16]){
 httpSection+=`#### ${concurrency === 1?'One connection':'16 connections'}\n\nRequests per second; higher is better.\n\n<div class="performance-table">\n\n| Workload | Tsrust | Node.js | Rust |\n| --- | ---: | ---: | ---: |\n`;
 for(const row of httpRows.filter(row=>row.concurrency===concurrency)){const n=row.engines.native,j=row.engines['node-tsc'],r=row.engines['rust-hand'];httpSection+=`| ${row.name} | ${rate(n.ops)} | ${rate(j.ops)} | ${rate(r.ops)} |\n`;}
 httpSection+='\n</div>\n\n';
}
httpSection+='SQLite uses 128 seeded in-memory rows and one connection: asynchronous SQLx for Tsrust/Rust, synchronous prepared statements for Node.js. These are warm reads, not disk throughput. Each batch validates all 128 responses. Builds, startup and seeding are excluded. The common Node client and response checks are included; this is not isolated server saturation.\n\n';
if(config.httpSqliteVersions)httpSection+=`SQLite versions: Tsrust ${cell(config.httpSqliteVersions.native)}, Node.js ${cell(config.httpSqliteVersions['node-tsc'])}, Rust ${cell(config.httpSqliteVersions['rust-hand'])}.\n\n`;
httpSection+='93 batch samples per engine and workload. Existing throughput baselines are reused. They do not contain request-latency or resource measurements.\n\n</details>\n\n';

const cpuManifest=JSON.parse(fs.readFileSync(path.join(root,'benchmarks/http/cpu-manifest.json'),'utf8'));
const cpuRows=[];
if(config.cpuBaselines){
 for(const [key,file] of [['oracle','cpu-oracle.json'],['oracleGenerator','cpu-oracle.py']])if(createHash('sha256').update(fs.readFileSync(path.join(root,'benchmarks/http',file))).digest('hex')!==cpuManifest.oracles[key])throw Error('Stale CPU oracle: '+file);
 const fingerprint=createHash('sha256').update(JSON.stringify({semanticEpoch:cpuManifest.semanticEpoch,protocol:cpuManifest.protocol,workloads:cpuManifest.workloads,oracles:cpuManifest.oracles})).digest('hex');
 if(fingerprint!==config.cpuProtocolSha256)throw Error('Stale CPU HTTP protocol');
 for(const workload of cpuManifest.workloads){
  const engines={};
  for(const [engine,history,file] of [['native',nativeRuns,'cpu.tsr'],['node-tsc',nodeRuns,'cpu.ts'],['rust-hand',rustRuns,'cpu.rs']]){
   const pin=config.cpuBaselines[workload.id]?.[engine];
   if(!pin||createHash('sha256').update(fs.readFileSync(path.join(root,'benchmarks/http',file))).digest('hex')!==pin.sourceSha256)throw Error('Stale CPU source: '+engine);
   const measured=history.get(workload.id)?.get(pin.at);
   if(!measured||measured.samples!==93||measured.requestSamples!==11904||measured.resourceRequests!==11904||!Number.isFinite(measured.requestP99Ms)||!Number.isFinite(measured.cpuMsPerRequest)||!(measured.peakRssBytes>0))throw Error('Missing CPU latency/resources: '+engine);
   engines[engine]=measured;
  }
  cpuRows.push({...workload,engines});
 }
}
let cpuSection='## HTTP benchmarks\n\n### CPU rendering\n\n';
cpuSection+='Rendering, simulation and numeric transformations spend time computing rather than waiting for I/O. This example parses a tile ID, renders a **96 × 64 Mandelbrot tile** with up to **256 iterations per pixel**, then returns its checksum and escape count as JSON. All three implementations perform the same arithmetic on varying inputs; an independent Python complex-number oracle checks every response.\n\n';
if(!cpuRows.length)cpuSection+='CPU request latency and resource measurements are being prepared; no result is claimed yet.\n\n';
else{
 const table=(row)=>{
  const headings=['Engine','req/s','p99 (ms)','CPU/req (ms)','RSS (MiB)'];
  let result='<div class="performance-table performance-metrics"><table role="table"><thead role="rowgroup"><tr role="row">'+headings.map(h=>'<th scope="col" role="columnheader">'+h+'</th>').join('')+'</tr></thead><tbody role="rowgroup">';
  for(const [id,label] of [['native','Tsrust'],['node-tsc','Node.js'],['rust-hand','Rust']]){
   const x=row.engines[id],values=[rate(x.ops),x.requestP99Ms.toFixed(2),x.cpuMsPerRequest.toFixed(2),(x.peakRssBytes/1048576).toFixed(1)];
   result+='<tr role="row"><th scope="row" role="rowheader">'+label+'</th>'+values.map((value,index)=>'<td role="cell" data-label="'+headings[index+1]+'">'+value+'</td>').join('')+'</tr>';
  }
  return result+'</tbody></table></div>\n\n';
 };
 const single=cpuRows.find(row=>row.concurrency===1),parallel=cpuRows.find(row=>row.concurrency===16);
 cpuSection+='#### One connection, one server worker\n\nHigher req/s is better. Lower p99, server CPU per request and peak sampled RSS are better. **Rust means hand-written Rust.**\n\n'+table(single);
 const gain=single.engines.native.ops/single.engines['node-tsc'].ops;
 cpuSection+=`For this kernel, Tsrust throughput is **${gain.toFixed(2)}× Node.js**, close to hand-written Rust. The larger difference is memory: **${(single.engines.native.peakRssBytes/1048576).toFixed(1)} MiB** peak sampled RSS versus **${(single.engines['node-tsc'].peakRssBytes/1048576).toFixed(1)} MiB** for Node.js. This is one workload, not a universal speedup claim.\n\n`;
 if(parallel)cpuSection+=`At 16 connections, observed request p99 was **${parallel.engines.native.requestP99Ms.toFixed(2)} ms** for Tsrust and **${parallel.engines['node-tsc'].requestP99Ms.toFixed(2)} ms** for Node.js.\n\n`;
 if(parallel)cpuSection+='<details>\n<summary>16 connections, same single-worker budget</summary>\n\n'+table(parallel)+'</details>\n\n';
 cpuSection+='<details>\n<summary>What the measurements mean</summary>\n\n';
 cpuSection+='One server process and one executor worker per engine; Rust is not given extra CPU workers. This is a single-machine, closed-loop loopback test with one common Node client, not a production p99 or a multicore scaling claim. Each engine runs three fresh processes, eight warmup batches and 31 timed batches of 128 requests. Throughput includes client verification.\n\n';
 cpuSection+='**p99** is the nearest-rank 99th percentile of **11,904 individual requests**, from request start until the complete response body arrives. It is not a percentile of batch durations. A 16-connection request can include queueing.\n\n';
 cpuSection+='**CPU/req** is total server-process CPU time divided by completed requests in a separate warmed resource pass. **RSS** is the highest sampled server resident set during that pass; it includes runtime/shared pages, excludes the load client, and is not heap usage or an allocation count. macOS ps reports CPU time at 10 ms resolution; RSS is polled every 50 ms, so short peaks can be missed. Resource polling is outside latency/throughput timing. Startup, compilation and oracle preparation are excluded.\n\n';
 cpuSection+='Sources, protocol and commands: `benchmarks/http/README.md`.\n\n</details>\n\n';
}


const highlightPin=config.cpuHighlight;
const highlight=rows.find(row=>row.workload===highlightPin?.workload);
if(!highlight||highlight.rust===undefined)throw Error('Missing measured CPU highlight');
const highlightWorkload=workloads.get(highlight.workload);
if(highlightWorkload.iterations!==highlightPin.iterations||manifest.seed!==highlightPin.seed)throw Error('CPU highlight measured inputs changed');
const highlightSource=fs.readFileSync(path.join(root,'benchmarks/runtime',highlightWorkload.source??('workloads/'+highlight.workload+'.tsr')),'utf8');
const highlightCounterpart=fs.readFileSync(path.join(root,'benchmarks/runtime',highlightWorkload.counterpart??('workloads/'+highlight.workload+'.ts')),'utf8');
if(createHash('sha256').update(highlightSource).digest('hex')!==highlightPin.sourceSha256||createHash('sha256').update(highlightCounterpart).digest('hex')!==highlightPin.counterpartSha256)throw Error('Stale CPU highlight source');
const highlightFunction=highlightSource.slice(0,highlightSource.indexOf('\nfunction main()')).trim();
if(!highlightFunction.startsWith('function workload('))throw Error('CPU highlight function boundary changed');
const highlightBatch=highlightWorkload.iterations.toLocaleString('en-US');
let highlightSection='### Selected CPU example: nested callbacks\n\n';
highlightSection+=`This microbenchmark measures **${highlightBatch} loop iterations**. Each iteration constructs nested callbacks, updates captured state and adds the returned value to a checksum. Tsrust measured **${(highlight.native/highlight.node).toFixed(2)}× Node.js throughput**; hand-written Rust measured **${(highlight.rust/highlight.node).toFixed(2)}×**.\n\n`;
const highlightCode='This is the actual measured Tsrust function:\n\n'+String.fromCharCode(96).repeat(3)+'tsrust\n'+highlightFunction+'\n'+String.fromCharCode(96).repeat(3)+'\n\n';
highlightSection+='<div class="performance-table">\n\n| Engine | Batch (ms) | vs Node |\n| --- | ---: | ---: |\n';
for(const [engine,label,at,value] of [['native','Tsrust',highlight.at,highlight.native],['node-tsc','Node.js',highlight.nodeAt,highlight.node],['rust-hand','Rust',highlight.rustAt,highlight.rust]]){
 const history=engine==='native'?nativeRuns:engine==='node-tsc'?nodeRuns:rustRuns;
 const measured=history.get(highlight.workload)?.get(at);
 if(!measured||measured.samples!==93)throw Error('CPU highlight measurement missing');
 highlightSection+=`| ${label} | ${measured.ms.toFixed(3)} | ${(value/highlight.node).toFixed(2)}× |\n`;
}
highlightSection+='\n</div>\n\nBatch time is the median; throughput ratios use mean iterations per second over all 93 samples. Each batch calls `workload('+highlightWorkload.iterations+', '+manifest.seed+')`, and its complete result is checked against an independent oracle. The TypeScript counterpart uses the same nested callbacks and exact values within its safe-integer range.\n\n';
highlightSection+=highlightCode;
highlightSection+='This is a selected callback microbenchmark. Its returned checksum retains the observable result; it does not establish a whole-application speedup. The full results below show the broader distribution, including workloads where Node.js wins. Existing accepted baselines are reused.\n\n';
const cpuSource=fs.readFileSync(path.join(root,'benchmarks/http/cpu.tsr'),'utf8');
const renderStart=cpuSource.indexOf('function render('),renderEnd=cpuSource.indexOf('\nasync function main()',renderStart);
if(renderStart<0||renderEnd<renderStart)throw Error('HTTP render function boundary changed');
cpuSection+='### The HTTP rendering function\n\n<details>\n<summary>Show the measured rendering function</summary>\n\n'+String.fromCharCode(96).repeat(3)+'tsrust\n'+cpuSource.slice(cpuSource.indexOf('type Output'),cpuSource.indexOf('type Ready')).trim()+'\n\n'+cpuSource.slice(renderStart,renderEnd).trim()+'\n'+String.fromCharCode(96).repeat(3)+'\n\nThe handler parses a JSON tile ID, calls `render(id)` and serializes the returned record. At one connection, its throughput gap is modest; its observed memory and tail-latency differences are reported alongside throughput.\n\n</details>\n\n';

let page = `---
title: "Performance"
description: "Measured Tsrust, Node.js and hand-written Rust execution speed."
---

Tsrust compiles TypeScript-style code to Rust executables. These measurements compare **Tsrust**, **TypeScript on Node.js**, and independently **hand-written Rust**.

## Built-in and language microbenchmarks

${highlightSection}### Overall results

This summary covers the language/library workloads, separately from the HTTP examples.

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

### Results by group

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
page += 'Each group lists every workload, slowest ratio first. Speeds are operations per second, abbreviated: k is thousands, M is millions and G is billions, so `5.35 M` is about 5,350,000 operations per second. “vs Node” is Tsrust throughput divided by Node.js throughput; above 1 means Tsrust was faster.\n\n';
for (const [group, entries] of ordered) {
  page += `#### ${group}\n\n<details>\n<summary>${entries.length} workloads</summary>\n\n<div class="performance-table">\n\n| Workload | Tsrust | Node.js | Rust | vs Node |\n| --- | ---: | ---: | ---: | ---: |\n`;
  for (const row of entries.sort((a, b) => ratio(a) - ratio(b))) {
    const label=cell(row.workload.replaceAll('_',' '));
    page += `| ${label} | ${rate(row.native)} | ${rate(row.node)} | ${row.rust === undefined ? "—" : rate(row.rust)} | ${ratio(row).toFixed(2)}× |\n`;
  }
  page += '\n</div>\n\n</details>\n\n';
}
page += `Rates are mean operations per second over every timed sample. Each workload
row is one complete workload: input setup, the operations and result retention
are timed together, so a rate is not the cost of one isolated operation.

${cpuSection}${httpSection}## Method

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
Run dates and source fingerprints are retained in repository metadata rather
than repeated in these tables. Missing Rust results are not inferred from Node.js timings.

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
