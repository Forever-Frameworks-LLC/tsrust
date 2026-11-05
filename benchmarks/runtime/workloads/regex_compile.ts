function workload(n: number, seed: number, input: string): number {
  let checksum = seed;
  let i = 0;
  while (i < n) {
    const expression = new RegExp(input, 'du');
    checksum += expression.test('a=12') ? 1 : 0;
    i++;
  }
  return checksum;
}
