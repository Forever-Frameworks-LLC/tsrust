type Handler = { run: (value: number) => number; };
function workload(n: number, seed: number): number {
    let checksum = seed;
    let i = 0;
    while (i < n) {
        const base = i + seed;
        const handler: Handler = {run: i % 2 === 0
            ? (value: number): number => { return base + value; }
            : (value: number): number => { return base - value; }};
        checksum = checksum + handler.run(i);
        handler.run = i % 3 === 0
            ? (value: number): number => { return value * 2; }
            : (value: number): number => { return value + 1; };
        checksum = checksum + handler.run(i);
        i = i + 1;
    }
    return checksum;
}
function main() { console.log(workload(2, 7)); }
