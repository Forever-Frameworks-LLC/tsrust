/// <reference types="node" />
import { Buffer } from "node:buffer";
export function workload(n: number, seed: number, input: string): number[] {
  let size = 32;
  if (input === "4k") { size = 4096; }
  if (input === "1m") { size = 1048576; }
  const text = "é😀Ab".repeat(size / 8);
  const bytes = Buffer.from(text, "utf8");
  const output: number[] = [];
  for (let i = 0; i < n; i += 1) {
    const result = bytes.toString("utf8");
    output.push(Buffer.byteLength(result, "utf8"));
    output.push(result.startsWith("é😀") ? 1 : 0);
    output.push(result.endsWith("Ab") ? 1 : 0);
  }
  return output;
}
