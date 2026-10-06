function main() { console.log(workload(2, 7)); }

function workload(n: number, seed: number): number {
    let checksum = seed;
    let i = 0;
    while (i < n) {
        const action = make(i + seed);
        checksum = checksum + action();
        i = i + 1;
    }
    return checksum;
}
function make<T>(value: T): () => T {
    return (): T => { return value; };
}
