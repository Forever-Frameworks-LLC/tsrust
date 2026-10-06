type int = number;
function workload(n: int, seed: int): string[] {
    const values: null[] = [null, null, null];
    const missing: undefined[] = [undefined, undefined, undefined];
    const output: string[] = []; let i = 0;
    while (i < n) {
        const count = (i + seed) % 4;
        let joined: string = "";
        if ((i + seed) % 2 === 0) {
            const receiver = values.slice(0, count);
            joined = receiver.toString();
        } else {
            const receiver = missing.slice(0, count);
            joined = receiver.toString();
        }
        output.push(joined);
        i = i + 1;
    }
    return output;
}
function main() { const result = workload(2, 7); }
