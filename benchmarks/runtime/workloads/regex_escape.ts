declare const Buffer: {byteLength(value: string, encoding: string): number};

function workload(n: number, seed: number, input: string): number {
  let checksum = seed;
  let i = 0;
  while (i < n) {
    const value = input.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
    checksum += Buffer.byteLength(value, 'utf8');
    i++;
  }
  return checksum;
}
