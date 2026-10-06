function workload(n: number, seed: number): number { let sum = 0; for (let i = 0; i < n; i++) { const action = (value: number): number => value + seed; sum += visit(action, 8) + countdown(counter(i), 4); } return sum; }
function visit(action: (value: number) => number, depth: number): number { return depth === 0 ? action(depth) : action(depth) + visit(action, depth - 1); }
function counter(start: number): () => number { let value = start; return () => ++value; }
function countdown(action: () => number, depth: number): number { if (depth === 0) return action(); const value = action(), moved = action; return value + countdown(moved, depth - 1); }
