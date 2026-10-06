type int = number;
type Point = { x: int; y: int; label: string; };
function pair(base: int, input: string): [int, string] { return [base + 1, input + "!"]; }
function workload(n: int, seed: int, input: string): string[][] {
 const output: string[][] = [];  let i = 0;
 while (i < n) { const base = (i * 13 + seed) % 97;
  const [count, word] = pair(base, input);
  const point: Point = { x: base, y: i % 7, label: input.slice() };
  const { x, y, label } = point;
  const pairs: [string, int][] = [["a", base], ["b", count]];
  let names = ""; let total = 0;
  for (const [name, amount] of pairs) { names += name; total += amount; }
  output.push([String(count + x + y), word + label, names, String(total)]); i = i + 1; } return output;
}
function main() { const result = workload(3, 0, "abc"); }
