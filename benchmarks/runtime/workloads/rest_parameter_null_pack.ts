type int = number;
function clone<T>(value: T): T { return value; }
function item(base: int, ordinal: int, input: string): null { return null; }
function describe(values: null[]): string[] { const result: string[] = []; let at = 0; while (at < values.length) { const kind = typeof values[at]; result.push(kind.concat(":", String(values[at]))); at = at + 1; } return result; }
function pack(...values: null[]): null[] { return values; }
function workload(n: int, seed: int, input: string): string[][] {
    const output: string[][] = []; let i = 0;
    while (i < n) { const mode = (i + seed) % 3; const base = (i * 13 + seed) % 97;
        const values = mode === 0 ? pack() : mode === 1 ? pack(item(base, 0, input)) : pack(item(base, 0, input), item(base, 1, input), item(base, 2, input), item(base, 3, input), item(base, 4, input));
        output.push(describe(values)); i = i + 1;
    } return output;
}
function main() { const result = workload(3, 0, "abc"); }
