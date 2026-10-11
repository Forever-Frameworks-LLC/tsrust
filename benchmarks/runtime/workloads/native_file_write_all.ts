declare function require(name: string): {
  openSync(path: string, flags: string): number;
  readSync(file: number, buffer: Uint8Array, offset: number, length: number, position: null): number;
  closeSync(file: number): void;
  writeSync(file: number, buffer: Uint8Array, offset: number, length: number, position: null): number;
};
const fileSystem = require("node:fs");
function observe(file: number, count: number): number[] {
  const bytes = new Uint8Array(count);
  const size = fileSystem.readSync(file, bytes, 0, count, null);
  let sum = 0;
  let index = 0;
  while (index < size) { sum += bytes[index]; index++; }
  return [size, sum];
}
function sample(input: string, count: number): number[] {
  const destination = input + ".out";
  const output = fileSystem.openSync(destination, "w");
  try {
    const owner = new TextEncoder().encode("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef");
    const bytes = owner.subarray(0, count);
    let written = 0;
    while (written < count) {
      const size = fileSystem.writeSync(output, bytes, written, count - written, null);
      if (size === 0) { throw Error("write-zero"); }
      written += size;
    }
  } finally { fileSystem.closeSync(output); }
  const file = fileSystem.openSync(destination, "r");
  try { return observe(file, count + 1); } finally { fileSystem.closeSync(file); }
}
function workload(n: number, seed: number, input: string): number[][] {
  const output: number[][] = [];
  let i = 0;
  while (i < n) {
    const count = (i + seed) % 64 + 1;
    output.push(sample(input, count));
    i++;
  }
  return output;
}
function main() {}
