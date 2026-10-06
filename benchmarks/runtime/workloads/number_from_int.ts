type int = number;
function workload(n: int, seed: int): number {
    let sum: number = 0.0; let i = 0;
    while (i < n) { const integer = (i * 37 + seed) % 2001 - 1000; sum = sum + Number(integer); i = i + 1; }
    return sum;
}
function main() { const result = workload(2, 7); console.log(result); }
