function workload(n: number, seed: number, input: string): string[] {
    const values: string[] = [clone(input), "", "é", "😀", "e\u0301", "a\u0000é😀", input.repeat(2), "Z"];
    const extras: number[] = [-10000, -1, 0, 1, 2, 5, 8, 17];
    const fillers: string[] = ["é😀e\u0301", "😀é", "aé😀", ""];
    const fillerLengths: number[] = [5, 3, 4, 0];
    const prefixLengths: number[] = [1, 2, 1, 0];
    const output: string[] = []; let i = 0;
    while (i < n) {
        const at = (i * 13 + seed) % 8;
        const fillAt = (i + seed) % 4;
        const tag = (i * 7 + seed) % 8;
        let extra: number = extras[tag];
        if (tag === 3) { extra = fillerLengths[fillAt]; }
        if (tag === 4) { extra = fillerLengths[fillAt] * 2; }
        if (tag === 5) { extra = fillerLengths[fillAt] + prefixLengths[fillAt]; }
        if (tag === 6) { extra = fillerLengths[fillAt] * 2 + prefixLengths[fillAt]; }
        if (tag === 7) { extra = prefixLengths[fillAt]; }
        if (fillAt === 3 && tag >= 3) { extra = 5; }
        const target: number = values[at].length + extra;
        const selected: string = values[at].padStart(target, fillers[fillAt]);
        output.push(selected);
        i = i + 1;
    }
    return output;
}
function main() { const result = workload(2, 7, " é😀é Z "); }
function clone<T>(value: T): T { return value; }
