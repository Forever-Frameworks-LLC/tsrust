function workload(n: number, seed: number, input: string): number {
    const values: string[] = ["", "a", "aa", "alpha", "alphabet", "z", input];
    let sum = seed;
    let i = 0;
    while (i < n) {
        const left = (i * 13 + seed) % 7;
        const right = (i * 17 + seed * 3) % 7;
        if (values[left] < values[right]) { sum = sum + 1; }
        if (values[(left + 1) % 7] <= values[(right + 2) % 7]) { sum = sum + 2; }
        if (values[(left + 2) % 7] > values[(right + 3) % 7]) { sum = sum + 4; }
        if (values[(left + 3) % 7] >= values[(right + 4) % 7]) { sum = sum + 8; }
        i = i + 1;
    }
    return sum;
}

function main() { console.log(workload(17, 7, "alpha")); }
