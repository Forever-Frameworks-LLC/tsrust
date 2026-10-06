// Explicit counterparts for the Rust-native UTF8/scalar contract.
const utf8Encoder = new TextEncoder();
function utf8Length(text: string): number { return utf8Encoder.encode(text).length; }
function rustTrim(text: string): string { return text.replace(/^\p{White_Space}+|\p{White_Space}+$/gu, ""); }
function workload(n: number, seed: number, input: string): number {
    const values = [input, rustTrim(input)];
    let digest = seed; let i = 0;
    while (i < n) {
        digest = (digest * 131 + utf8Length(values[(i + seed) % 2])) % 1000000007;
        i = i + 1;
    }
    return digest;
}
function main() { console.log(workload(2, 7, " A😀e\u0301 Z ")); }
