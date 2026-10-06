type int = number;
function workload(n: int, seed: int): number {
    const table0: number[] = [-3.5, -1.5, -0.5, -0.125, 0.125, 0.5, 1.5, 3.5];
    const table1: number[] = [0.25, 0.5, 0.75, 1.0, 1.25, 1.5, 2.0, 3.5];
    let sum: number = Number(seed); let i = 0;
    while (i < n) {
        const position = (i * 13 + seed) % 8;
        const jitter = (i * 17 + seed) % 5;
        const argument0: number = table0[position] + Number(jitter) * 0.03125;
        const argument1: number = table1[position] + Number(jitter) * 0.015625;
        sum = sum + operation(argument0, argument1);
        i = i + 1;
    }
    return sum;
}
function operation(argument0: number, argument1: number): number { return Math.atan2(argument0, argument1); }
function main() { const result = workload(2, 7); console.log(result); }
