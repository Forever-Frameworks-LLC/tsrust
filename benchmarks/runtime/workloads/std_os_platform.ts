declare function require(name: string): { platform(): string };
const platform = require("node:os").platform;
function workload(n: number, seed: number): string[] {
 const values: string[] = []; let i = 0;
 while (i < n) { values.push(platform()); i = i + 1; }
 return values;
}
