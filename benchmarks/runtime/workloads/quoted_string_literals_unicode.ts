type int = number;
function workload(n: int, seed: int, input: string): string[] {
    const values: string[] = [clone(input), 'é😀', "BOM\uFEFF中", '\xE9', "\u00E9\u4E2D", '\u{1F600}\u{10FFFF}', "\uD83D\uDE00", '\u0065\u0301', '\qé\$\_', "\0é😀", 'é\
😀', "é\😀", 'é\
😀', "é\ 😀", 'é\ 😀', ""];
    const output: string[] = []; let i = 0;
    while (i < n) {
        const at = (i * 13 + seed) % 16;
        const selected: string = clone(values[at]);
        output.push(selected);
        i = i + 1;
    }
    return output;
}
function main() { const result = workload(2, 7, "dynamic é😀"); }
function clone<T>(value: T): T { return value; }
