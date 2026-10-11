declare function require(name: string): { performance: { now(): number } };
const clock = require("node:perf_hooks").performance;
function workload(n: number, seed: number): boolean[] {
  const values: boolean[] = [];
  let previous = clock.now();
  let i = 0;
  while (i < n) {
    const current = clock.now();
    values.push(Number.isFinite(current) && current >= previous && current >= 0);
    previous = current;
    i = i + 1;
  }
  return values;
}
