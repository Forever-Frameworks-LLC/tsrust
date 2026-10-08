declare function require(name: string): { posix: { isAbsolute(path: string): boolean } };
const isAbsolute = require("node:path").posix.isAbsolute;
function workload(n: number, seed: number): string[] {
    const values: string[] = [];
    let i = 0;
    while (i < n) {
        let path = "/tmp/report.txt";
        if ((i + seed) % 2 === 0) { path = "reports/../archive/"; }
        values.push(isAbsolute(path).toString());
        i = i + 1;
    }
    return values;
}
