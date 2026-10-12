function workload(n: number, seed: number, input: string): number {
  let checksum = seed;
  let i = 0;
  while (i < n) {
    const value: unknown = JSON.parse(input);
    if (typeof value !== 'number' || !Number.isSafeInteger(value)) { throw 'integer'; }
    checksum += value;
    if (typeof value !== 'number' || value !== 42.0) { throw 'number'; }
    const boolean: unknown = JSON.parse('true');
    if (typeof boolean !== 'boolean' || !boolean) { throw 'boolean'; }
    const string: unknown = JSON.parse('"hello"');
    if (typeof string !== 'string' || string !== 'hello') { throw 'string'; }
    i++;
  }
  return checksum;
}
