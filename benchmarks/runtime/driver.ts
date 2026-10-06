// Appended to each independently authored workload; timed bodies exclude reporting.
const host = globalThis as unknown as { Deno?: { args: string[] }; process?: { argv: string[] } };
const argv = host.Deno ? host.Deno.args : host.process!.argv.slice(2);
const iterations = Number(argv[0]);
const seed = Number(argv[1]);
const warmups = Number(argv[2]);
const samples = Number(argv[3]);
let expected: number | undefined;
function validate(output: number | string[] | number[] | number[][]): void {
  let value: number;
  if (typeof output === "number") value = output;
  else {
    value = 0;
    for (const item of output) {
      if (typeof item === "number") value = (value * 131 + item) % 1000000007;
      else if (typeof item === "string") {
        for (const scalar of item) { const code = scalar.codePointAt(0)!; if (code >= 0xD800 && code <= 0xDFFF) throw new Error("Invalid Unicode output"); value = (value * 131 + code) % 1000000007; }
        value = (value * 131 + 255) % 1000000007;
      } else {
        for (const number of item) value = (value * 131 + number) % 1000000007;
        value = (value * 131 + 255) % 1000000007;
      }
    }
  }
  if (!Number.isSafeInteger(value)) throw new Error('Checksum outside safe integer domain');
  if (expected !== undefined && value !== expected) throw new Error('Unstable checksum');
  expected = value;
}
for (let i = 0; i < warmups; i++) validate(workload(iterations, seed));
const durations: number[] = [];
for (let i = 0; i < samples; i++) {
  const start = performance.now();
  const value = workload(iterations, seed);
  durations.push(performance.now() - start);
  validate(value);
}
console.log(JSON.stringify({checksum: expected, durations}));
