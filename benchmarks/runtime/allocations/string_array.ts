function workload(n: number, seed: number): string[] { const values: string[] = []; for (let i=0; i<n; i++) values.push("Pat"); return values; }
function probeChecksum(value: unknown): number { return (value as string[]).reduce((sum, text) => sum + [...text].reduce((s,c) => s + c.charCodeAt(0), 0), 0); }
