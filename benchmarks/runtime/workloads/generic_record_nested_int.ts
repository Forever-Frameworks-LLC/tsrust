type Box<T> = { value: T; };

function workload(n: number, seed: number): number {
    let checksum = seed;
    let i = 0;
    while (i < n) {
        const item: Box<Box<number>> = { value: { value: (i + seed) % 1009 } };
        checksum = checksum + item.value.value;
        i = i + 1;
    }
    return checksum;
}

function main() { console.log(workload(2, 7)); }
