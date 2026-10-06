// Declared natural result timing wrapper; complete schema observation happens after elapsed time.
const host = globalThis as unknown as { Deno?: { args: string[] }; process?: { argv: string[] } };
const argv = host.Deno ? host.Deno.args : host.process!.argv.slice(2);
const iterations = Number(argv[0]), seed = Number(argv[1]);
const warmups = Number(argv[2]), samples = Number(argv[3]);
let expected: {checksum: number; exactBits?: string} | undefined;
function validateDeclaredResult(output: unknown): void {
    const observed = observeDeclared(output, __DECLARED_RESULT__);
    if (!Number.isSafeInteger(observed.checksum)) throw new Error('Unsafe checksum');
    if (expected && (expected.checksum !== observed.checksum || expected.exactBits !== observed.exactBits)) throw new Error('Unstable exact number result');
    expected = observed;
}
for (let i = 0; i < warmups; i++) validateDeclaredResult(workload(iterations, seed));
const durations: number[] = [];
for (let i = 0; i < samples; i++) {
    const start = performance.now(), value = workload(iterations, seed);
    durations.push(performance.now() - start);
    validateDeclaredResult(value);
}
console.log(JSON.stringify({...expected, durations}));
