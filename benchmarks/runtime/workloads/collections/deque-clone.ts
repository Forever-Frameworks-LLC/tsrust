type Item = { values: number[]; };
function workload(n: number, seed: number, input: string): number[][] {
  const output: number[][] = [];
  for (let i = 0; i < n; i++) {
    const value = (i + seed) % 256;
    const original: Item[] = [];
    original.push({values: [value, 255]});
    const copied = original.map(item => ({values: item.values.slice()}));
    const changed = copied.shift()!;
    changed.values.push(value + 1);
    const retained = original.shift()!;
    output.push([retained.values.length, retained.values[0], changed.values.length, changed.values[2], original.length, copied.length]);
  }
  return output;
}
function main() {}
