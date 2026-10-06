function workload(n: number, seed: number): number { const action = counter(seed); let sum = 0; for (let i = 0; i < n; i++) sum += action(); return sum; }
function counter(start: number): () => number { let value = start; return () => ++value; }
