function workload(n: number, seed: number, input: string): number {
  let checksum = seed;
  let i = 0;
  while (i < n) {
    const value = JSON.parse(input);
    if (typeof value !== 'boolean') { throw 'Expected boolean'; }
    if (value) { checksum += 1; }
    i++;
  }
  return checksum;
}
