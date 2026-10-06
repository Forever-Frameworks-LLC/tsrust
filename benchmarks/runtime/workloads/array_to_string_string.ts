type int = number;
function workload(n: int, seed: int, input: string): string[] {
    const values: string[] = [clone(input), "", "é😀e\u0301", "\uFEFF\u0000", input.repeat(2)];
    const output: string[] = []; let i = 0;
    while (i < n) {
        const count = (i + seed) % 6;
        let joined: string = "";
        const receiver = values.slice(0, count);
        joined = receiver.toString();
        output.push(joined);
        i = i + 1;
    }
    return output;
}
function main() { const result = workload(2, 7, "dynamic text"); }
function clone<T>(value: T): T { return value; }
