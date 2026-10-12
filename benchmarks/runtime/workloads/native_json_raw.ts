function workload(n: number, seed: number, input: string): number {
  let checksum = seed;
  let i = 0;
  while (i < n) {
    JSON.parse(input);
    checksum += 1;
    i++;
  }
  return checksum;
}
