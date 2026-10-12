function workload(n: number, seed: number, input: string): number {
  let checksum = seed;
  let i = 0;
  while (i < n) {
    const value = JSON.parse(input);
    if (typeof value !== 'string') { throw 'Expected string'; }
    if (!value.startsWith('hello') || !value.endsWith('hello') || !value.includes('ell')) { throw 'Expected matches'; }
    checksum += value.length + value.indexOf('hello') + value.lastIndexOf('hello');
    i++;
  }
  return checksum;
}
