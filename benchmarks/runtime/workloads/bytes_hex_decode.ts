/// <reference types="node" />
import { Buffer } from "node:buffer";
function fromHex(text: string): Buffer { if (text.length % 2 !== 0 || !/^[0-9a-fA-F]*$/.test(text)) throw Error("hex syntax"); return Buffer.from(text, "hex"); }
export function workload(n: number, seed: number, input: string): number[] {
  let size = 32;
  if (input === "4k") { size = 4096; }
  if (input === "1m") { size = 1048576; }
  const text = "é😀Ab".repeat(size / 8);
  const encoded = Buffer.from(text, "utf8").toString("hex");
  const output: number[] = [];
  for (let i = 0; i < n; i += 1) {
    const result = fromHex(encoded);
    output.push(result.length);
    output.push(result[0]);
    output.push(result[result.length - 1]);
  }
  return output;
}
