function workload(n: number, seed: number, input: string, coordinates: number): string[] {
    const width = coordinates; const results: string[] = []; let i = 0;
    while (i < n) {
        results.push(input.charAt((i * 13 + seed) % (width * 2 + 3) - width));
        i = i + 1;
    }
    return results;
}
function main() { const results = workload(2, 7, "ASCIIxyz", 8); console.log(results.length); }
