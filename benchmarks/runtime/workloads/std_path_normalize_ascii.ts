declare function require(name: string): { posix: { normalize(path: string): string } };
const normalize = require("node:path").posix.normalize;
function workload(n: number, seed: number): string[] {
    const values: string[] = [];
    let i = 0;
    while (i < n) {
        let path = "/tmp//reports/../report.txt";
        if ((i + seed) % 2 === 0) { path = "reports/../archive/"; }
        values.push(normalize(path));
        i = i + 1;
    }
    return values;
}
