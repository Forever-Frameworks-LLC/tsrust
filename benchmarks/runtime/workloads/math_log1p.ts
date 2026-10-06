type int = number;
function workload(n: int, seed: int): number {
    const table0: number[] = [-0.375, -0.25, -0.125, -0.0625, 0.0625, 0.125, 0.25, 0.75];
    let sum: number = Number(seed); let i = 0;
    while (i < n) {
        const position = (i * 13 + seed) % 8;
        const jitter = (i * 17 + seed) % 5;
        const argument0: number = table0[position] + Number(jitter) * 0.0078125;
        sum = sum + operation(argument0);
        i = i + 1;
    }
    return sum;
}
function operation(argument0: number): number { return Math.log1p(argument0); }
function main() { const result = workload(2, 7); console.log(result); }
