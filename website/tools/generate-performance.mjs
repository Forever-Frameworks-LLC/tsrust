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

let page = `---
title: "Performance"
description: "Measured native execution speed compared with the same programs compiled by tsc and run on Node.js."
---

These measurements compare a Tsrust program compiled to native code with an
equivalent TypeScript program compiled by \`tsc\` and run on Node.js. Both sides
compute the same checked result for each workload. Higher operations per second
is faster; the ratio is native throughput divided by Node.js throughput, so a ratio
above 1 means the native program was faster.

This snapshot uses the latest accepted measurement selected for each workload.
Rows carry their measurement date; unchanged workloads retain earlier results.
Rejected experiments and candidates still in validation are excluded. These
results compare native execution with Node.js. Hand-written Rust baselines are
shown where implemented and measured; a dash means coverage is still missing.

## Summary

| Measure | Value |
| --- | --- |
| Workloads compared | ${rows.length} |
| Rust faster than Node.js | ${rows.filter(row => row.rust !== undefined && row.rust > row.node).length} of ${rows.length} (${rows.filter(row => row.rust !== undefined).length} measured) |
| Native faster | ${faster} of ${rows.length} |
| Median ratio (native ÷ Node.js) | ${median.toFixed(2)}× |
| Range | ${ratios[0].toFixed(2)}× to ${ratios.at(-1).toFixed(2)}× |
| Machine | ${cell(machine.cpu ?? 'unrecorded')} (${cell(machine.platform ?? '')} ${cell(machine.architecture ?? '')}) |
| Toolchains | Rust ${cell(config.rust)}; Node.js ${cell(config.node)}; TypeScript ${cell(config.typescript)} |
| Samples | ${samples} timed samples per engine and workload |
| Measured | ${runs.map((run) => run.timestamp.slice(0, 10)).filter((value, index, all) => all.indexOf(value) === index).join(', ')} |

## Results by group

`;
const middle = (values) => { const sorted = [...values].sort((a, b) => a - b); const half = sorted.length >> 1; return sorted.length % 2 ? sorted[half] : (sorted[half - 1] + sorted[half]) / 2; };
const ordered = [...groups].sort(([a], [b]) => a.localeCompare(b));
page += '| Group | Workloads | Native faster | Median ratio |\n| --- | ---: | ---: | ---: |\n';
for (const [group, entries] of ordered) {
  const values = entries.map(ratio);
  page += `| [${group}](#${group.toLowerCase().replaceAll(/[^a-z0-9]+/g, '-')}) | ${entries.length} | ${values.filter((value) => value > 1).length} | ${middle(values).toFixed(2)}× |\n`;
}
const slower = ordered.filter(([, entries]) => middle(entries.map(ratio)) < 1).map(([group]) => `the ${group} group`);
page += slower.length
  ? `\nNative code is not uniformly faster. In ${slower.join(' and ')}, the median workload runs slower than on Node.js: these remain optimization targets. A throughput table alone does not identify the cause of a performance gap.\n\n`
  : '\n';
page += 'Each group lists every workload, slowest ratio first. Speeds are operations per second, abbreviated: k is thousands, M is millions and G is billions, so `5.35 M` is about 5,350,000 operations per second. The ratio column is the one to compare.\n\n';
for (const [group, entries] of ordered) {
  page += `### ${group}\n\n<details>\n<summary>${entries.length} workloads</summary>\n\n| Workload | Native (ops/s) | Node.js (ops/s) | Native / Node | Hand-written Rust (ops/s) | Native / Rust | Native date | Node date | Rust date |\n| --- | ---: | ---: | ---: | ---: | ---: | --- | --- | --- |\n`;
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

- **Native**: the Tsrust compiler emits Rust, built with the pinned Rust
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
- Native strings use UTF-8 semantics. Where a workload depends on string encoding,
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
every program runs faster natively. Short workloads are sensitive to ordinary
machine variation, and small ratio differences are not significant. Rows where
Node.js is faster are kept and reviewed rather than removed.

The measurements are in \`benchmarks/measurements.sqlite\` in the repository:
one row per workload and engine, with each run's date, median sample time,
mean speed and sample count. It can be queried with the standard SQLite tools.
The benchmark runner and every workload source pair are in
\`benchmarks/runtime\`.
`;
fs.mkdirSync(path.dirname(output), { recursive: true });
fs.writeFileSync(output, page);
console.log(`Wrote ${path.relative(root, output)} with ${rows.length} comparisons from ${runs.length} run(s).`);
