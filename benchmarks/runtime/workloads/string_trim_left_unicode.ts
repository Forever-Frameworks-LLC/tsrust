function workload(n: number, seed: number, input: string): string[] {
    const values: string[] = [clone(input), input.repeat(2), input.repeat(3), input.repeat(1), clone(input), "", "   \u2028  \u2029  　", "   \u2028 literal é😀é \u2029  　"];
    const output: string[] = []; let i = 0;
    while (i < n) {
        output.push(values[(i * 13 + seed) % 8].trimLeft());
        i = i + 1;
    }
    return output;
}
function main() { const result = workload(2, 7, " é😀é Z "); }
function clone<T>(value:T):T{return value;}
