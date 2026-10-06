function workload(n: number, seed: number, input: string): string[] {
    const values: string[] = [clone(input), input.repeat(2), input.repeat(1), "", "é\u0000😀", " é ", "﻿", clone(input)];
    const output: string[] = []; let i = 0;
    while (i < n) {
        let value: string = clone(values[(i * 13 + seed) % 8]);
        value += values[(i * 7 + seed + 3) % 8];
        output.push(value);
        i = i + 1;
    }
    return output;
}
function main() { const result = workload(2, 7, " é😀é Z "); }
function clone<T>(value: T): T { return value; }
