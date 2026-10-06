function workload(n: number, seed: number): number {
 let state = seed, sum = seed;
 for (let i = 0; i < n; i++) {
  state = (state * 17 + 13) % 1009;
  sum += state % 2 === 0 && state > 100 ? state : -1;
 } return sum;
}
