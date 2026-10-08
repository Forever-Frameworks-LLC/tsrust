declare function require(name: string): { join(...paths: string[]): string };
const join = require("node:path").join;
function workload(n: number, seed: number): string[] {
    const values: string[] = []; let i = 0;
    while (i < n) { let part = "reports"; if ((i + seed) % 2 === 0) { part = ".."; } values.push(join("/tmp", part, "archive.txt")); i = i + 1; }
    return values;
}
