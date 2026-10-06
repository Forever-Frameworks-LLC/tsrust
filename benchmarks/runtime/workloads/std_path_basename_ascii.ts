declare function require(name: string): { posix: { basename(path: string, suffix?: string): string } };
const pathModule = require("node:path").posix;

function workload(n: number, seed: number): string[] {
    const values: string[] = [];
    let i = 0;
    while (i < n) {
        let path = "/tmp/report.txt";
        if ((i + seed) % 2 === 0) {
            path = "/var/log/archive.txt/";
        }
        values.push(pathModule.basename(path, ".txt"));
        i = i + 1;
    }
    return values;
}
