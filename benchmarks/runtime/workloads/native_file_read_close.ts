declare function require(name: string): {
  openSync(path: string, flags: string): number;
  readSync(file: number, buffer: Uint8Array, offset: number, length: number, position: null): number;
  closeSync(file: number): void;
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
  const file = fileSystem.openSync(input, "r");
  try { return observe(file, count); } finally { fileSystem.closeSync(file); }
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
