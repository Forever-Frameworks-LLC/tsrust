function workload(n: number, seed: number, input: string): string[] {
    const width = input.length; const results: string[] = []; let i = 0;
    while (i < n) {
        results.push(input.slice((i * 13 + seed) % (width * 3 + 1) - width, (i * 7 + seed) % (width * 3 + 1) - width));
        i = i + 1;
    }
    return results;
}
function main() { const results = workload(2, 7, "ASCIIxyz"); console.log(results.length); }
