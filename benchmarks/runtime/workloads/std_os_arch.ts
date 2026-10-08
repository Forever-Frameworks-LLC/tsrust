declare function require(name: string): { arch(): string };
const arch = require("node:os").arch;
function workload(n: number, seed: number): string[] {
 const values: string[] = []; let i = 0;
 while (i < n) { values.push(arch()); i = i + 1; }
 return values;
}
