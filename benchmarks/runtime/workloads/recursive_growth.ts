type Tree = {value: number; children: Tree[]};
function workload(n: number, seed: number): number { let sum = seed; for (let i = 0; i < n; i++) { const root: Tree = {value: i, children: []}; const width = (i + seed) % 33; for (let j = 0; j < width; j++) root.children.push({value: seed + j, children: []}); if (width) sum += root.children.splice(Math.floor(width / 2), 1)[0].value; sum += total(root); } return sum; }
function total(tree: Tree): number { let sum = tree.value; for (const child of tree.children) sum += total(child); return sum; }
