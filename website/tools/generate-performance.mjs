// Generates the public performance page from the benchmark SQLite dataset.
// The runs to publish are listed in performance-runs.json by raw SHA-256, so the
// page changes only when that list or the accepted measurements change.
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { DatabaseSync } from 'node:sqlite';

const here = path.dirname(fileURLToPath(import.meta.url));
const root = path.resolve(here, '../..');
const config = JSON.parse(fs.readFileSync(path.join(here, 'performance-runs.json'), 'utf8'));
const manifest = JSON.parse(fs.readFileSync(path.join(root, 'benchmarks/runtime/manifest.json'), 'utf8'));
const workloads = new Map(manifest.workloads.map((workload) => [workload.id, workload]));
const output = path.join(root, 'website/src/content/docs/project/performance.md');

const db = new DatabaseSync(path.join(root, 'benchmarks/measurements.sqlite'), { readOnly: true });
const rows = [];
const runs = [];
for (const raw of config.runs) {
  const run = db.prepare("SELECT timestamp, metadata FROM analytics_runs WHERE raw_sha = ? AND kind = 'execution'").get(raw);
  if (!run) throw Error(`Run ${raw} is not in benchmarks/measurements.sqlite`);
  runs.push({ raw, timestamp: run.timestamp, metadata: JSON.parse(run.metadata) });
  const query = db.prepare(`SELECT w.workload, w.iterations, w.operations_per_iteration,
      n.avg_ops_per_second AS native, n.samples AS native_samples,
      j.avg_ops_per_second AS node, j.samples AS node_samples
    FROM analytics_workloads w
    JOIN analytics_summaries n USING (raw_sha, workload, source_sha)
    JOIN analytics_summaries j USING (raw_sha, workload, source_sha)
    WHERE w.raw_sha = ? AND n.engine = 'native' AND j.engine = 'node-tsc'
    ORDER BY w.workload`);
  for (const row of query.all(raw)) rows.push(row);
}
db.close();
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
const first = runs[0].metadata;
const machine = first.machine ?? {};
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

## Summary

| Measure | Value |
| --- | --- |
| Workloads compared | ${rows.length} |
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
  ? `\nNative code is not uniformly faster. In ${slower.join(' and ')}, the median workload runs slower than on Node.js: those programs are dominated by owned record and string copies and by checked decoding, which Node.js handles with shared references and a tuned JIT. They are current optimization targets.\n\n`
  : '\n';
page += 'Each group lists every workload, slowest ratio first. Speeds are operations per second, abbreviated: k is thousands, M is millions and G is billions, so `5.35 M` is about 5,350,000 operations per second. The ratio column is the one to compare.\n\n';
for (const [group, entries] of ordered) {
  page += `### ${group}\n\n<details>\n<summary>${entries.length} workloads</summary>\n\n| Workload | Native (ops/s) | Node.js (ops/s) | Ratio |\n| --- | ---: | ---: | ---: |\n`;
  for (const row of entries.sort((a, b) => ratio(a) - ratio(b))) {
    const label = row.description ? `\`${row.workload}\`: ${cell(row.description)}` : `\`${row.workload}\``;
    page += `| ${label} | ${rate(row.native)} | ${rate(row.node)} | ${ratio(row).toFixed(2)}× |\n`;
  }
  page += '\n</details>\n\n';
}
page += `Rates are mean operations per second over every timed sample. Each workload
row is one complete workload: input setup, the operations and result retention
are timed together, so a rate is not the cost of one isolated operation.

## Method

- **Native**: the Tsrust compiler emits Rust, built with the pinned Rust
  toolchain at optimization level 3 with integer overflow checks enabled.
- **Node.js**: the matching TypeScript program is compiled once with \`tsc\` and
  run with the pinned Node.js version. JIT warm-up runs before timing.
- Each engine runs ${config.processes} separate processes with ${config.warmups} warm-up
  batches and ${config.samplesPerProcess} timed samples per process. Startup and
  compilation time are excluded.
- Every correctness case is checked against an independent oracle before timing,
  and the timed result is verified afterwards.
- Native strings use UTF-8 semantics. Where a workload depends on string encoding,
  the TypeScript counterpart performs the same UTF-8 work explicitly.

## Limits

These are workload-level comparisons on one machine, not a general claim that
every program runs faster natively. Short workloads are sensitive to ordinary
machine variation, and small ratio differences are not significant. Rows where
Node.js is faster are kept and reviewed rather than removed.

The raw samples are in \`benchmarks/measurements.sqlite\` in the repository,
keyed by run and source hash, and can be queried with the standard SQLite tools.
The benchmark runner and every workload source pair are in
\`benchmarks/runtime\`.
`;
fs.mkdirSync(path.dirname(output), { recursive: true });
fs.writeFileSync(output, page);
console.log(`Wrote ${path.relative(root, output)} with ${rows.length} comparisons from ${runs.length} run(s).`);
