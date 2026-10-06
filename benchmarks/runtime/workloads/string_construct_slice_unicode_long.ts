// Explicit counterparts for the Rust-native UTF8/scalar contract.
const utf8Encoder = new TextEncoder();
const utf8Decoder = new TextDecoder("utf-8", {fatal: true, ignoreBOM: true});
function utf8Length(text: string): number { return utf8Encoder.encode(text).length; }
function utf8Slice(text: string, start: number, end: number): string {
 const bytes = utf8Encoder.encode(text);
 const bound = (index: number): number => index < 0 ? Math.max(0, bytes.length + index) : Math.min(bytes.length, index);
 const first = bound(start), last = bound(end);
 return last <= first ? "" : utf8Decoder.decode(bytes.subarray(first, last));
}
function workload(n: number, seed: number, input: string): string[] {
    const width = utf8Length(input); const results: string[] = []; let i = 0;
    while (i < n) {
        const start = (i + seed) % 8;
        const end = width - 1 - (i * 7 + seed) % 8;
        results.push(utf8Slice(input, start, end));
        i = i + 1;
    }
    return results;
}
function main() { const results = workload(2, 7, "ABCDEFGHA😀é中Σabcdefgh"); console.log(results.length); }
