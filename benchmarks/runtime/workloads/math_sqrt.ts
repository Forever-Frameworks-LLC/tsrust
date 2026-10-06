type int = number;
function workload(n: int, seed: int): number {
    const values: number[] = [0.0, 0.25, 1.0, 2.0, 3.0, 6.25, 10.0, 25.0];
    let sum: number = Number(seed); let i = 0;
    while (i < n) {
        const value = values[(i * 13 + seed) % 8];
        sum = sum + Math.sqrt(value);
        i = i + 1;
    }
    return sum;
}
function main() { const result = workload(2, 7); console.log(result); }
