type int = number;
function workload(n: int, seed: int): number {
    let sum: number = Number(seed); let i = 0;
    while (i < n) {
        const a: number = Number((i * 13 + seed) % 65 - 32) * 0.125;
        const b: number = Number((i * 17 + seed) % 81 - 40) * 0.25;
        const c: number = Number((i * 19 + seed) % 97 - 48) * 0.5;
        const value_result: number = Math.min(a, b, c);
        sum = sum + value_result;
        i = i + 1;
    }
    return sum;
}
function main() { const result = workload(2, 7); console.log(result); }
