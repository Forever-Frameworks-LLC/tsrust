function workload(n: number, seed: number): number { let sum = 0; for (let i = 0; i < n; i++) { const action = counter(i, 8); sum += action() + action(); const lazy = chain(seed, 4); sum += lazy(); } return sum; }
function counter(start: number, depth: number): () => number { if (depth > 0) return counter(start, depth - 1); let value = start; return () => ++value; }
function chain(seed: number, depth: number): () => number { return () => { if (depth === 0) return seed; const next = chain(seed, depth - 1); return next(); }; }
