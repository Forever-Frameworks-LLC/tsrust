declare function require(name: string): { join(...paths: string[]): string };
const join = require("node:path").join;
function workload(n: number, seed: number): string[] {
    const values: string[] = []; let i = 0;
    while (i < n) { let first = ""; if ((i + seed) % 2 === 0) { first = "/"; } values.push(join(first, "", "child", "..", "")); i = i + 1; }
    return values;
}
