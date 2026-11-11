function workload(n: number, seed: number, input: string): number {
  let checksum = seed;
  let i = 0;
  while (i < n) {
    const value = new URL(input);
    const params = new URLSearchParams(value.search);
    const rows = Array.from(params.entries(), ([name, value]) => ({name, value}));
    params.set('tag', 'next');
    params.append('q', 'a b+');
    params.sort();
    const encoded = params.toString();
    if (encoded !== 'q=a+b%2B&tag=next&z=last' || rows.length !== 3 || rows[0].name !== 'tag' || rows[0].value !== 'one' || rows[1].name !== 'tag' || rows[1].value !== 'two' || rows[2].name !== 'z' || rows[2].value !== 'last') {
      throw 'Invalid query snapshot';
    }
    value.search = params.toString();
    params.delete('tag');
    const result = value.search;
    if (result.length !== 25 || !result.startsWith('?q=a+b%2B&tag=next&z=last')) {
      throw 'Invalid query replacement';
    }
    checksum += encoded.length + result.length + rows[0].value.length + rows[1].value.length + rows[2].value.length;
    i++;
  }
  return checksum;
}
