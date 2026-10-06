type int = number;
function workload(n: int, seed: int): boolean[] {
    const values: boolean[] = [true, false];
    const output: boolean[] = []; let i = 0;
    while (i < n) {
        const mode = (i + seed) % 3;
        const at = (i * 13 + seed) % 2;
        const value: boolean | null | undefined = mode === 0 ? null : mode === 1 ? undefined : values[at];
        if (value !== null && value !== undefined) { output.push(true); output.push(value.valueOf()); } else if (value === null) { output.push(false); output.push(false); } else { output.push(false); output.push(true); }
        i = i + 1;
    }
    return output;
}
function main() { const result = workload(2, 7); }
