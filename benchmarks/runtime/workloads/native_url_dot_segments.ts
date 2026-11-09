function workload(n: number, seed: number, input: string): number {
  const size = input.length * 8192;
  const authority = input.startsWith('L') ? 'a'.repeat(size) : 'a';
  const text = 'https://' + authority + '@example.com/' + '../'.repeat(size) + 'x';
  const expected = 'https://' + authority + '@example.com/x';
  let checksum = seed;
  let i = 0;
  while (i < n) {
    const value = new URL(text);
    const href = value.href;
    if (href.length !== expected.length || !href.startsWith(expected) || value.username.length !== authority.length || value.pathname.length !== 2 || !value.pathname.startsWith('/x')) {
      throw 'Invalid dot-segment serialization';
    }
    checksum += href.length + value.username.length + value.pathname.length;
    i++;
  }
  return checksum;
}
