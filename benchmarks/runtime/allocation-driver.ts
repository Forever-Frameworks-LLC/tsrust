// Node-only retained heap experiment. Forced GC is excluded from runtime timing.
declare const process: {argv: string[]; memoryUsage(): {heapUsed: number}};
declare function gc(): void;
const n = Number(process.argv[2]);
// Prime code/measurement paths, then release the trial result before baseline GC.
let result: unknown = workload(0, 7);
result = undefined;
process.memoryUsage(); gc(); gc();
const before = process.memoryUsage().heapUsed;
result = workload(n, 7);
gc(); gc();
const after = process.memoryUsage().heapUsed;
const checksum = probeChecksum(result);
// Keep the result alive throughout the measurement and checksum.
(globalThis as unknown as {probeResult: unknown}).probeResult = result;
console.log(JSON.stringify({checksum, heapUsedBefore: before, heapUsedAfter: after, retainedHeapDelta: after - before}));
