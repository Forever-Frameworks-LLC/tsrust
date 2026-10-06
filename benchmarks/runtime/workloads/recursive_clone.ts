type Tree = { value: number; children: Tree[] };
function workload(n: number, seed: number): number[][] {
 const roots: Tree[] = [];
 for (let i = 0; i < n; i++) {
  const children: Tree[] = [];
  for (let j = 0; j < (i + seed) % 5; j++) children.push({value: (3 * i + j + seed) % 1009, children: []});
  let tree: Tree = {value: i % 1009, children};
  for (let depth = 0; depth < (i + seed) % 4; depth++) tree = {value: (i + depth + seed) % 1009, children: [tree]};
  roots.push(tree);
 }
 const copy = (tree: Tree): Tree => ({value: tree.value, children: tree.children.map(copy)});
 const copied = roots.map(copy);
 for (let index = 0; index < n; index++) {
  roots[index].value = index + 2000;
  if (roots[index].children.length) roots[index].children[0].value = seed + 3000;
 }
 const output: number[][] = [];
 const project = (tree: Tree, row: number[]): void => { row.push(tree.value); for (const child of tree.children) project(child, row); };
 for (const tree of copied) { const row: number[] = []; project(tree, row); output.push(row); }
 for (const tree of roots) { const row: number[] = []; project(tree, row); output.push(row); }
 return output;
}
