type int = number;
function workload(n: int, seed: int): number[][] {
    const values: number[] = [Number(seed) + 0.25, -0.0, 2.25, Number(seed) + 1.5, 0.0, 4.5, 5.25, -2.5];
    const starts: int[] = [0, 0, 1, -1, -8, -80, 0, 1, 3, -3, 100, -100, 3, -2, -2, 0];
    const ends: int[] = [0, 0, 0, 0, 0, 0, 0, 3, 3, -1, 200, -80, 1, -2, 200, -1];
    const modes: int[] = [0, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2];
    const output: number[][] = []; let i = 0;
    while (i < n) {
        const query = (i * 13 + seed) % 16;
        const mode = modes[query];
        if (mode === 0) { output.push(values.slice()); }
        else {
            if (mode === 1) { output.push(values.slice(starts[query])); }
            else { output.push(values.slice(starts[query], ends[query])); }
        }
        i = i + 1;
    }
    return output;
}
function main() { const result = workload(2, 7); console.log(result.length); }
