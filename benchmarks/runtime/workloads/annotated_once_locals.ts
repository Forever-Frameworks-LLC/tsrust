function workload(n: number, seed: number): number {
    let checksum = seed;
    let i = 0;
    while (i < n) {
        const base = i + seed;
        const action: (value: number) => number = (value: number): number => {
            return base + value;
        };
        checksum = checksum + action(i);
        i = i + 1;
    }
    return checksum;
}
function main() { console.log(workload(2, 7)); }
