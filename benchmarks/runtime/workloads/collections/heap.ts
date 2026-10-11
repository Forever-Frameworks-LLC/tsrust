function sumItems(...items: number[]): number { let total = 0; for (const item of items) { total += item; } return total; }
function workload(n: number, seed: number, input: string): number[][] {
  const output: number[][] = [];
  for (let i = 0; i < n; i++) {
    const values: number[] = [];
    for (let j = 0; j < 9; j++) { values.push((i + seed + j) % 256); }
    values.sort((a, b) => a - b);
    let traversed = 0;
    for (const value of values) { traversed += value; }
    const spread = sumItems(...values);
    const flattened = [...values];
    const top = values[values.length - 1];
    const copied = values.slice();
    let sum = 0;
    while (copied.length > 0) { sum += copied.pop()!; }
    output.push([values.length, top, sum, copied.length, traversed, spread, flattened.length]);
  }
  return output;
}
function main() {}
