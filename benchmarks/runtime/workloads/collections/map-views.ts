type int = number;
function byteWindow(owner: Uint8Array): Uint8Array { return owner.subarray(0, 2); }
function first(part: Uint8Array): int { return part[0]; }
function fill(values: Map<int, Uint8Array>, owner: Uint8Array): void {
  let key = 0;
  while (key < 8) { values.set(key, byteWindow(owner)); key++; }
}
function collect(owner: Uint8Array): Map<int, Uint8Array> {
  const values = new Map<int, Uint8Array>();
  fill(values, owner);
  return values;
}
function exposed(owner: Uint8Array): Uint8Array {
  const values = collect(owner);
  const selected = values.get(0);
  if (selected !== undefined) { return selected.subarray(0, 2); }
  throw "missing";
}
function workload(n: int, seed: int, input: string): int[][] {
  const output: int[][] = [];
  let i = 0;
  while (i < n) {
    const byte = (i + seed) % 256;
    const owner = Uint8Array.from([byte, 255]);
    const values = collect(owner);
    let keys = 0;
    let sum = 0;
    for (const [key, part] of values) { keys += key; sum += first(part); }
    const part = exposed(owner);
    output.push([values.size, keys, sum, first(part), part.length]);
    i++;
  }
  return output;
}
