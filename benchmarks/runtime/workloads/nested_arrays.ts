function workload(n: number, seed: number): number {
 let sum = seed;
 for (let i = 0; i < n; i++) {
  const rows = [[i, seed], [seed, i + 1]]; rows[1].push(i + 2); rows[0][0] = i + 3;
  const saved = rows.map(row => row.slice()); rows[0][0] = 0;
  for (const row of saved) for (const value of row) sum += value;
 } return sum;
}
