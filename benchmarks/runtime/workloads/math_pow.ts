type int = number;
function workload(n: int, seed: int): number {
    const table0: number[] = [0.5, 0.625, 0.75, 1.125, 1.5, 1.75, 2.25, 2.5];
    const table1: number[] = [-2.5, -1.75, -1.25, -0.625, 0.375, 0.75, 1.25, 2.5];
    let sum: number = Number(seed); let i = 0;
    while (i < n) {
        const position = (i * 13 + seed) % 8;
        const jitter = (i * 17 + seed) % 5;
        const argument0: number = table0[position] + Number(jitter) * 0.015625;
        const argument1: number = table1[position] + Number(jitter) * 0.0078125;
        sum = sum + operation(argument0, argument1);
        i = i + 1;
    }
    return sum;
}
function operation(argument0: number, argument1: number): number { return Math.pow(argument0, argument1); }
function main() { const result = workload(2, 7); console.log(result); }
