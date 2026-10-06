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
function scalarCharAt(text: string, index: number): string {
 if (index < 0) return "";
 let position = 0;
 for (const scalar of text) { if (position === index) return scalar; position++; }
 return "";
}
function workload(n: number, seed: number, input: string): string[] {
 const boundaries: number[] = [0]; let position = 0; let bytes = 0;
 while (true) { const scalar = scalarCharAt(input, position); if (scalar === "") { break; } bytes = bytes + utf8Length(scalar); boundaries.push(bytes); position = position + 1; }
 const width = boundaries.length - 1; const totalBytes = utf8Length(input); const results: string[] = []; let i = 0;
 while (i < n) {
  const start = byteCoordinate(boundaries, totalBytes, width, (i * 13 + seed) % (width * 3 + 1) - width);
  const end = byteCoordinate(boundaries, totalBytes, width, (i * 7 + seed) % (width * 3 + 1) - width);
  results.push(utf8Slice(input, start, end)); i = i + 1;
 } return results;
}
function byteCoordinate(boundaries: number[], totalBytes: number, width: number, ordinal: number): number {
 if (ordinal < 0) { const index = width + ordinal; if (index < 0) { return -totalBytes; } return boundaries[index] - totalBytes; }
 if (ordinal > width) { return totalBytes; } return boundaries[ordinal];
}
function main() { const results = workload(2, 7, "A😀e\u0301Z"); console.log(results.length); }
