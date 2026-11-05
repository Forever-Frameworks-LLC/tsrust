function workload(n: number, seed: number, input: string): number {
  const value = new URL(input);
  let checksum = seed;
  let i = 0;
  while (i < n) {
    value.pathname = '/changed/path';
    value.search = '?q=one+two';
    value.port = '8443';
    const href = value.href;
    if (href.length !== 47 || !href.startsWith('https://example.com:8443/changed/path?q=one+two')) {
      throw 'Invalid URL setters';
    }
    checksum += href.length;
    i++;
  }
  return checksum;
}
