function workload(n: number, seed: number, input: string): string[] {
    const values: string[] = [clone(input), "", "é", "😀", "e\u0301", "a\u0000é😀", input.repeat(2), "Z"];
    const output: string[] = []; let i = 0;
    while (i < n) {
        const at = (i * 13 + seed) % 8;
        const selected: string = values[at].concat();
        output.push(selected);
        i = i + 1;
    }
    return output;
}
function main() { const result = workload(2, 7, " é😀é Z "); }
function clone<T>(value: T): T { return value; }
