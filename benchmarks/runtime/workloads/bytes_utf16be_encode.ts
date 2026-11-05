/// <reference types="node" />
import { Buffer } from "node:buffer";
function fromUtf16(text: string, order: string): Buffer { const bytes = Buffer.from(text, "utf16le"); return order === "big" ? bytes.swap16() : bytes; }
export function workload(n: number, seed: number, input: string): number[] {
  let size = 32;
  if (input === "4k") { size = 4096; }
  if (input === "1m") { size = 1048576; }
  const text = "é😀Ab".repeat(size / 8);

  const output: number[] = [];
  for (let i = 0; i < n; i += 1) {
    const result = fromUtf16(text, "big");
    output.push(result.length);
    output.push(result[0]);
    output.push(result[result.length - 1]);
  }
  return output;
}
