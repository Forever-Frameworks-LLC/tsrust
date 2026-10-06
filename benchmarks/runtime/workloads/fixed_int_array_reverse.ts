function workload(n: number, seed: number): number {
    let sum = seed; let i = 0;
    while (i < n) {
        const values: number[] = [seed + i, seed + i + 1, seed + i + 2, seed + i + 3, seed + i + 4, seed + i + 5, seed + i + 6, seed + i + 7, seed + i + 8];
        values.reverse();
        let j = 0;
        while (j < values.length) { sum = (sum * 131 + values[j]) % 1000000007; j = j + 1; }
        i = i + 1;
    } return sum;
}
function main() { console.log(workload(2, 7)); }
