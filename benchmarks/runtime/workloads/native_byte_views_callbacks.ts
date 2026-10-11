type Picker = { pick: (other: Uint8Array) => Uint8Array; };
function window(owner: Uint8Array, start: number, end: number): Uint8Array {
  return owner.subarray(start, end);
}
function first(value: Uint8Array): number { return value[0]; }
function make(captured: Uint8Array, expected: number): Picker {
  return { pick: (other: Uint8Array) => {
    const kept = captured;
    if (first(kept) !== expected) { throw "capture mismatch"; }
    return other;
  } };
}
export function workload(n: number, seed: number, input: string): number[][] {
  const output: number[][] = [];
  let i = 0;
  while (i < n) {
    const byte = (i + seed) % 256;
    const next = (byte + 1) % 256;
    const owner = Uint8Array.from([byte, next, 255]);
    const picker = make(window(owner, 0, 1), byte);
    const selected = picker.pick(window(owner, 1, 3));
    output.push([selected.length, first(selected)]);
    i++;
  }
  return output;
}
