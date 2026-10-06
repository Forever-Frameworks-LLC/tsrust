type int = number;
function searchResult(input: string, needle: string, position?: number): boolean {
    return input.endsWith(needle, position);
}
function workload(n: int, seed: int, input: string): int {
    if (Array.from(input).some(scalar => scalar.codePointAt(0)! > 127)) throw new Error('ASCII input required');
    const needles: string[] = ["", "A", "alpha", "Z", "missing\u0000", "Aalpha", "a"];
    const positions: int[] = [-7, 0, 1, 3, 7, 8, 9, 11, 15, 16, 18, 22, 23, 999];
    let sum = seed; let i = 0;
    while (i < n) {
        const position = positions[(i * 13 + seed) % 14];
        let code = 0;
        if ((i + seed) % 2 === 0) {
            if (searchResult(input, needles[(i * 17 + seed) % 7])) { code = 1; }
        } else {
            if (searchResult(input, needles[(i * 17 + seed) % 7], position)) { code = 1; }
        }
        sum = (sum * 131 + code) % 1000000007;
        i = i + 1;
    } return sum;
}
function main() { console.log(workload(17, 7, "AalphaAalphaZ")); }
