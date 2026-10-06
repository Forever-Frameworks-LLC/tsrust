function workload(n: number, seed: number): number[] { const values: number[] = []; for (let i=0; i<n; i++) values.push(seed + i); return values; }
function probeChecksum(value: unknown): number { return (value as number[]).reduce((sum, v) => sum + v, 0); }
