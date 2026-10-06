function workload(n: number, seed: number): number { let sum = 0; for (let i = 0; i < n; i++) { let action = counter(i); sum += action(); action = counter(seed); const moved = action; sum += moved(); } return sum; }
function counter(start: number): () => number { let value = start; return () => ++value; }
