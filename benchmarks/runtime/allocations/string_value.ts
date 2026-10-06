function workload(n: number, seed: number): string { return "Pat"; }
function probeChecksum(value: unknown): number { const text = value as string; return [...text].reduce((sum, c) => sum + c.charCodeAt(0), 0); }
