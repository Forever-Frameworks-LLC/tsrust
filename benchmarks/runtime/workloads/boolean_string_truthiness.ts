type int = number;
function workload(n: int, seed: int, input: string): boolean[] {
    const values: string[] = ["", input.repeat(1), "false", "0", "\u0000", "é😀"];
    const output: boolean[] = []; let i = 0;
    while (i < n) {
        output.push(Boolean(values[(i + seed) % 6]));
        i = i + 1;
    }
    return output;
}
function main() { const result = workload(2, 7, "dynamic text"); }
