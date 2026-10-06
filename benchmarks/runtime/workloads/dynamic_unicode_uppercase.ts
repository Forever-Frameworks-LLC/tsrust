function workload(n: number, seed: number, input: string): string[] {
    const results: string[] = [];
    let i = 0;
    while (i < n) {
        results.push(input.toUpperCase());
        i = i + 1;
    }
    return results;
}
function main() { const results = workload(2, 7, "Straße ﬀi e\u0301 😀\u0000Z"); console.log(results.length); }
