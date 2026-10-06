function workload(n: number, seed: number): number[] { const output: number[] = []; for (let index = 0; index < n; index++) output.push((17 * index + seed) % 1009); return output; }
