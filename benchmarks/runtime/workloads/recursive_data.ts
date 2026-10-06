type Tree = {value: number; children: Tree[]};
function workload(n: number, seed: number): number { let sum = 0; for (let i = 0; i < n; i++) { const tree: Tree = {value: i, children: [{value: seed, children: []}, {value: i + 1, children: []}]}; sum += total(tree) + even(8, seed); } return sum; }
function total(tree: Tree): number { let result = tree.value; for (const child of tree.children) result += total(child); return result; }
function even(depth: number, seed: number): number { return depth === 0 ? seed : depth + odd(depth - 1, seed); }
function odd(depth: number, seed: number): number { return depth === 0 ? seed : depth + even(depth - 1, seed); }
