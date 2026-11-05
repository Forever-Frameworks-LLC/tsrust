/// <reference types="node" />
import { Buffer } from "node:buffer";
export function base64Decode(text: string, url: boolean): Buffer {
  const padding = text.endsWith("==") ? 2 : text.endsWith("=") ? 1 : 0;
  const end = text.length - padding;
  if ((url && padding !== 0) || (!url && text.length % 4 !== 0) || end % 4 === 1) throw Error("base64 padding");
  let last = 0;
  for (let i = 0; i < end; i += 1) {
    const code = text.charCodeAt(i);
    if (code >= 65 && code <= 90) last = code - 65;
    else if (code >= 97 && code <= 122) last = code - 71;
    else if (code >= 48 && code <= 57) last = code + 4;
    else if (code === (url ? 45 : 43)) last = 62;
    else if (code === (url ? 95 : 47)) last = 63;
    else throw Error("base64 alphabet");
  }
  const remainder = end % 4;
  if ((!url && padding !== (remainder === 2 ? 2 : remainder === 3 ? 1 : 0)) || (remainder === 2 && (last & 15) !== 0) || (remainder === 3 && (last & 3) !== 0)) throw Error("base64 trailing bits");
  return Buffer.from(text, url ? "base64url" : "base64");
}
export function workload(n: number, seed: number, input: string): number[] {
  let size = 32;
  if (input === "4k") { size = 4096; }
  if (input === "1m") { size = 1048576; }
  const text = "é😀Ab".repeat(size / 8);
  const encoded = Buffer.from(text, "utf8").toString("base64");
  const output: number[] = [];
  for (let i = 0; i < n; i += 1) {
    const result = base64Decode(encoded, false);
    output.push(result.length);
    output.push(result[0]);
    output.push(result[result.length - 1]);
  }
  return output;
}
