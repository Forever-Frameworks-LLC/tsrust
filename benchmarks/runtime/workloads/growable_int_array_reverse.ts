function workload(n: number, seed: number): number {
    let sum = seed; let i = 0;
    while (i < n) {
        const values: number[] = []; let k = 0; while (k < 9) { values.push(seed + i + k); k = k + 1; }
        values.reverse();
        let j = 0;
        while (j < values.length) { sum = (sum * 131 + values[j]) % 1000000007; j = j + 1; }
        i = i + 1;
    } return sum;
}
function main() { console.log(workload(2, 7)); }
