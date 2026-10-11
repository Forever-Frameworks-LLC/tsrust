type Chunk = { data: Uint8Array; children: Chunk[] };
function byteWindow(owner: Uint8Array): Uint8Array { return owner.subarray(0, 2); }
function first(chunk: Chunk): number { return chunk.data[0]; }
function workload(n: number, seed: number, input: string): number[][] {
  const output: number[][] = [];
  let i = 0;
  while (i < n) {
    const byte = (i + seed) % 256;
    const owner = Uint8Array.from([byte, 255]);
    let root: Chunk = { data: byteWindow(owner), children: [] };
    let depth = 0;
    while (depth < 16) {
      root = { data: byteWindow(owner), children: [root] };
      depth++;
    }
    let sum = 0;
    let count = 0;
    while (root.children.length > 0) {
      sum += first(root);
      count++;
      const child = root.children.pop()!;
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
