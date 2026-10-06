type int = number;
function workload(n: int, seed: int): boolean[] {
    const values: boolean[] = [false, true, true, false];
    const output: boolean[] = []; let i = 0;
    while (i < n) {
        const at = (i * 13 + seed) % 4;
        const selected: boolean = values[at].valueOf();
        output.push(selected);
        i = i + 1;
    }
    return output;
}
function main() { const result = workload(2, 7); }
