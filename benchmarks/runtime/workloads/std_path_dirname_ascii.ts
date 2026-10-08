declare function require(name: string): { posix: { dirname(path: string): string } };
const dirname = require("node:path").posix.dirname;
function workload(n: number, seed: number): string[] {
    const values: string[] = [];
    let i = 0;
    while (i < n) {
        let path = "/tmp/report.txt";
        if ((i + seed) % 2 === 0) { path = "/var/log/archive.tar.gz/"; }
        values.push(dirname(path));
        i = i + 1;
    }
    return values;
}
