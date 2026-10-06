type int = number;
function workload(n: int, seed: int, input: string): string[][] {
    const values: string[] = [clone(input), "", "::a::::b::", "é😀é😀", "a\u0000b", "e\u0301::😀", "\uFEFF::中", input.repeat(2)];
    const limits: int[] = [0, 1, 2, 3, 7];
    const unitLimits: int[][] = values.map(text => limits.map(limit => scalarUnitLimit(text, limit)));
    const output: string[][] = []; let i = 0;
    while (i < n) {
        const at = (i * 13 + seed) % 8;
        const limitIndex = (i * 3 + seed) % 5;
        const pieces: string[] = mergeScalarPieces(values[at].split("", unitLimits[at][limitIndex]));
        output.push(pieces);
        i = i + 1;
    }
    return output;
}
function main() { const result = workload(2, 7, "é::😀::"); }
function clone<T>(value: T): T { return value; }
function scalarUnitLimit(source: string, limit: int): int {
    if (limit === 0) { return 0; }
    let scalars = 0; let units = 0;
    for (const scalar of source) { if (scalars >= limit) { break; } units += scalar.length; scalars += 1; }
    return units;
}
function mergeScalarPieces(parts: string[]): string[] {
    const pieces: string[] = []; let i = 0;
    while (i < parts.length) {
        const high = parts[i].charCodeAt(0);
        if (high >= 55296 && high <= 56319 && i + 1 < parts.length) { pieces.push(parts[i] + parts[i + 1]); i += 2; }
        else { pieces.push(parts[i]); i += 1; }
    }
    return pieces;
}
