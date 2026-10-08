declare function require(name: string): { posix: { extname(path: string): string } };
const extname = require("node:path").posix.extname;
function workload(n: number, seed: number): string[] {
    const values: string[] = [];
    let i = 0;
    while (i < n) {
        let path = "/tmp/report.txt";
        if ((i + seed) % 2 === 0) { path = "/var/log/archive.tar.gz/"; }
        values.push(extname(path));
        i = i + 1;
    }
    return values;
}
