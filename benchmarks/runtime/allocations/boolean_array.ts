function workload(n: number, seed: number): boolean[] { const values: boolean[] = []; for (let i=0; i<n; i++) values.push(i % 2 === 0); return values; }
function probeChecksum(value: unknown): number { return (value as boolean[]).filter(v => v).length; }
