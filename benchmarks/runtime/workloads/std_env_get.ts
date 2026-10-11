declare function require(name: string): { env: Record<string, string | undefined> };
const environment = require("node:process").env;
function workload(n: number, seed: number): string[] {
 const names = ["TSRUST_BENCH_ENV_TEXT", "TSRUST_BENCH_ENV_EMPTY", "TSRUST_BENCH_ENV_MISSING"];
 const values: string[] = [];
 let i = 0;
 while (i < n) {
  const value = environment[names[(i + seed) % 3]];
  values.push(value === undefined ? "<missing>" : value);
  i = i + 1;
 }
 return values;
}
