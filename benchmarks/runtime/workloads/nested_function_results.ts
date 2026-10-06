function workload(n: number, seed: number, input: string): number[][] {
    const output: number[][] = [];
    let i = 0;
    while (i < n) {
        const value = i % 5 + seed % 5 + 1;
        function offset(extra: number) { return value + extra; }
        const values = pair(value);
        output.push([double(value), identity(value + 3), offset(2), values[0], values[1], input.length]);
        i += 1;
    }
    return output;
}
function double(value: number) { return value + value; }
function identity<T>(value: T) { return value; }
function pair(value: number) { return [value, value + 1]; }
function main() {}
