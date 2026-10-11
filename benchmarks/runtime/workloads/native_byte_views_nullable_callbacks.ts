type Chunk = { read?: () => Uint8Array; next?: Chunk };
function byteWindow(owner: Uint8Array): Uint8Array { return owner.subarray(0, 2); }
function first(data: Uint8Array): number { return data[0]; }
function node(owner: Uint8Array): Chunk { const captured=byteWindow(owner); return {read:()=>captured}; }
function alternate(owner: Uint8Array): Chunk { const captured=byteWindow(owner); return {read:()=>captured}; }
function empty():Chunk{return {};}

function take(chunk: Chunk) { return chunk.read; }

function workload(n: number, seed: number, input: string): number[][] {
  const output: number[][] = [];
  let i = 0;
  while (i < n) {
    const byte = (i + seed) % 256;
    const owner = Uint8Array.from([byte, 255]);
    let root = empty();
    let depth = 0;
    while (depth < 16) {
      const parent = depth % 2 === 0 ? node(owner) : alternate(owner); parent.next = root; root = parent;
      depth++;
    }
    let sum = 0;
    let count = 0;
    while (root.next !== undefined) {
      sum += first(root.read!());
      count++;
      const child = root.next!;
      root = child;
    }
    const tail = take(root);
    const absent = tail === undefined ? 1 : 0;
    count++;
    output.push([count, sum, absent]);
    i++;
  }
  return output;
}
function main() {}
