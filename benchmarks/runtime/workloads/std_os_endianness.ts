declare function require(name: string): { endianness(): string };
const endianness = require("node:os").endianness;
function workload(n: number, seed: number): string[] {
 const values: string[] = []; let i = 0;
 while (i < n) { values.push(endianness()); i = i + 1; }
 return values;
}
