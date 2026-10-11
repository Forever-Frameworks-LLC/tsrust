type Chunk = { data: Uint8Array; read: () => Uint8Array; next?: Chunk };
function byteWindow(owner: Uint8Array): Uint8Array { return owner.subarray(0, 2); }
function first(data: Uint8Array): number { return data[0]; }
function node(owner: Uint8Array): Chunk { const data=byteWindow(owner); const captured=byteWindow(owner); return {data:data,read:()=>captured}; }
function workload(n: number, seed: number, input: string): number[][] {
  const output: number[][] = [];
  let i = 0;
  while (i < n) {
    const byte = (i + seed) % 256;
    const owner = Uint8Array.from([byte, 255]);
    let root = node(owner);
    let depth = 0;
    while (depth < 16) {
      const parent = node(owner); parent.next = root; root = parent;
      depth++;
    }
    let sum = 0;
    let count = 0;
    while (root.next !== undefined) {
      sum += first(root.read());
      count++;
      const child = root.next!;
      root = child;
    }
    sum += first(root.read());
    count++;
    output.push([count, sum, root.data.length]);
    i++;
  }
  return output;
}
function main() {}
