type Chunk = { data: Uint8Array; next?: Chunk };
function byteWindow(owner: Uint8Array): Uint8Array { return owner.subarray(0, 2); }
function first(chunk: Chunk): number { return chunk.data[0]; }
function workload(n: number, seed: number, input: string): number[][] {
  const output: number[][] = [];
  let i = 0;
  while (i < n) {
    const byte = (i + seed) % 256;
    const owner = Uint8Array.from([byte, 255]);
    let root: Chunk = { data: byteWindow(owner) };
    let depth = 0;
    while (depth < 16) {
      root = { data: byteWindow(owner), next: root };
      depth++;
    }
    let sum = 0;
    let count = 0;
    while (root.next !== undefined) {
      sum += first(root);
      count++;
      const child = root.next!;
      root = child;
    }
    sum += first(root);
    count++;
    output.push([count, sum, root.data.length]);
    i++;
  }
  return output;
}
function main() {}
