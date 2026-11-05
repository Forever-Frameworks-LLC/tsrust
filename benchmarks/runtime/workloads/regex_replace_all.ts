function workload(n: number, seed: number, input: string): number {
  const expression = new RegExp('([a-z]+)=([0-9]+)', 'dgu');
  let checksum = seed;
  let i = 0;
  while (i < n) {
    expression.lastIndex = 0;
    const value = input.replace(expression, '$2:$1 $$');
    checksum += value.length;
    i++;
  }
  return checksum;
}
