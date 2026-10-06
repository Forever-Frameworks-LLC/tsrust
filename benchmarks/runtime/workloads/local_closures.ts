function workload(n: number, seed: number): number {
 let sum = seed;
 for (let i = 0; i < n; i++) { let state = i;
  const outer = (): number => { const inner = (): number => { state += 2; return state; }; return inner() + seed; };
  sum += outer();
 } return sum;
}
