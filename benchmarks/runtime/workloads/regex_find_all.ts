function workload(n: number, seed: number, input: string): number {
  const expression = new RegExp('([a-z]+)=([0-9]+)', 'gu');
  let checksum = seed;
  let i = 0;
  while (i < n) {
    expression.lastIndex = 0;
    const values = Array.from(input.matchAll(expression), value => ({start: value.index, end: value.index + value[0].length}));
    for (const value of values) { checksum += value.start + value.end + 1; }
    i++;
  }
  return checksum;
}
