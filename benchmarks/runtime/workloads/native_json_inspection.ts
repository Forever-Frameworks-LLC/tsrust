function workload(n: number, seed: number, input: string): number {
  let checksum = seed;
  let i = 0;
  while (i < n) {
    const value = JSON.parse(input);
    if (!Array.isArray(value) || value === null) { throw 'Expected array'; }
    if (JSON.parse('null') !== null) { throw 'Expected null'; }
    checksum += 2;
    i++;
  }
  return checksum;
}
