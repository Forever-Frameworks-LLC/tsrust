type int = number;
function workload(n: int, seed: int, input: string): string[] {
    const values: string[] = [input.repeat(1), "", "static", "é😀e\u0301", "\uFEFF\u0000"];
    const output: string[] = []; let i = 0;
    while (i < n) {
        const mode = (i + seed) % 3;
        const at = (i * 13 + seed) % 5;
        const initial: string | undefined = mode === 0 ? undefined : values[at].repeat(1);
        const value: string | undefined = pass(initial);
        output.push(typeof value);
        output.push(String(value));
        output.push(Boolean(value) ? "true" : "false");
        output.push(value === undefined ? "sentinel" : "payload");
        i = i + 1;
    }
    return output;
}
function pass(value: string | undefined): string | undefined { return value; }
function main() { const result = workload(2, 7, "dynamic text"); }
