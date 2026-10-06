type int = number;
function workload(n: int, seed: int, input: string): string[][] {
 const output: string[][] = [];  let i = 0;
 while (i < n) { const mode = (i + seed) % 3; const text = input.repeat(mode);
 const letters = [...text]; output.push([letters.at(i % 7) ?? "none", letters.at(-1 - mode) ?? "none"]); i = i + 1; } return output;
}
function main() { const result = workload(3, 0, "abc"); }
