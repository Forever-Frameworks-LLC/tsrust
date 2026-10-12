function workload(n: number, seed: number, input: string): number {
  let checksum = seed;
  let i = 0;
  while (i < n) {
    const text = JSON.stringify(JSON.parse(input));
    if (text !== '{"a":[1,true,null],"b":"hello"}') {
      throw "Unexpected JSON output";
    }
    checksum += text.length;
    i++;
  }
  return checksum;
}
