function workload(n: number, seed: number, input: string): number {
  const expression = new RegExp('([a-z]+)=([0-9]+)', 'dgu');
  let checksum = seed;
  let i = 0;
  while (i < n) {
    expression.lastIndex = 0;
    checksum += expression.test(input) ? 1 : 0;
    i++;
  }
  return checksum;
}
