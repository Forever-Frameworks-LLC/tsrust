type int = number;
function workload(n: int, seed: int): string[] {
    const values: number[] = [Number(seed), -0.0, 0.5, -2.5, 17.0, Number.NaN, Number.POSITIVE_INFINITY, Number.NEGATIVE_INFINITY];
    const output: string[] = []; let i = 0;
    while (i < n) {
        const count = (i + seed) % 9;
        let joined: string = "";
        const receiver = values.slice(0, count);
        joined = receiver.toString();
        output.push(joined);
        i = i + 1;
    }
    return output;
}
function main() { const result = workload(2, 7); }
