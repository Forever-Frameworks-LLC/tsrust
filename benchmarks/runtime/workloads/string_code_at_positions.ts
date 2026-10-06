type int = number;
function workload(n: int, seed: int, input: string): number[][] {
 const output: number[][] = [];  let i = 0;
 while (i < n) { const mode = (i + seed) % 3; const position = (i * 13 + seed) % 9; const text = "aé€z".repeat(mode + 1);
 output.push([Number(text.codePointAt(position) ?? -1), text.charCodeAt(position), Number(text.codePointAt(position + 20) ?? -1), text.charCodeAt(-1 - mode), Number(text.codePointAt(mode) ?? -1)]); i = i + 1; } return output;
}
function main() { const result = workload(3, 0, "abc"); }
