type int = number;
function workload(n: int, seed: int): number[][] {
    const values: number[] = [Number(seed) + 0.25, -0.0, 2.25, Number(seed) + 1.5, 0.0, 4.5, 5.25, -2.5];
    const output: number[][] = []; let trace = seed; let visits = 0; let i = 0;
    while (i < n) {
        const selected: number[] = values.filter((value: number): boolean => {
            trace = (trace * 131 + visits + i + seed) % 1000000007; visits = visits + 1;
            return value <= Number((i + seed) % 5);
        });
        output.push(selected); i = i + 1;
    }
    output.push([Number(trace), Number(visits)]);
    return output;
}
function main() { const result = workload(2, 7); console.log(result.length); }
