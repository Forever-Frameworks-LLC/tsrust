// Explicit counterparts for the Rust-native UTF8/scalar contract.
function scalarCharAt(text: string, index: number): string {
 if (index < 0) return "";
 let position = 0;
 for (const scalar of text) { if (position === index) return scalar; position++; }
 return "";
}
function workload(n: number, seed: number, input: string, coordinates: number): string[] {
    const width = coordinates; const results: string[] = []; let i = 0;
    while (i < n) {
        results.push(scalarCharAt(input, width - 7 + (i * 13 + seed) % 11));
        i = i + 1;
    }
    return results;
}
function main() { const results = workload(2, 7, "ABC😀Z", 5); console.log(results.length); }
