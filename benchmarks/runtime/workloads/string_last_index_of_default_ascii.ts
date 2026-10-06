type int = number;
function workload(n: int, seed: int, input: string): int[] {
    const needles: string[] = ["", "a", "aa", "aba", "alpha", "A", "Z", "missing\u0000", "a\u0000b"];
    const output: int[] = []; let i = 0;
    while (i < n) {
        const needle = needles[(i * 17 + seed) % 9];
        const at = input.lastIndexOf(needle);
        output.push(at + 1); i = i + 1;
    }
    return output;
}
function main() { const result = workload(2, 7, "aaa ababa alpha A alpha Z"); }
