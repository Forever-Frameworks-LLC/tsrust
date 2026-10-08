const relative = require("node:path").relative;

function workload(n: number, seed: number): string[] {
    const values: string[] = [];
    const from = "reports/é😊";
    let i = 0;
    while (i < n) {
        let to = "reports/é😊/2026";
        if ((i + seed) % 2 === 0) {
            to = "reports/archive.txt";
        }
        values.push(relative(from, to));
        i = i + 1;
    }
    return values;
}

function main() {
    const values = workload(2, 7);
    for (const value of values) {
        console.log(value);
    }
}
