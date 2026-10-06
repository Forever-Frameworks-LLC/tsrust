function workload(n: number, seed: number): number {
 let sum = seed;
 for (let i = 0; i < n; i++) {
  const values: number[] = []; for (let j = 0; j < 32; j++) values.push(seed + i + j);
  values[3] = i; const saved = values.slice();
  for (const value of saved) { if (value === seed) continue; if (value > 20000) break; sum += value; }
  const flags = [true, false]; flags[1] = i % 2 === 0; if (flags[1]) sum += flags.length - 1;
 } return sum;
}
