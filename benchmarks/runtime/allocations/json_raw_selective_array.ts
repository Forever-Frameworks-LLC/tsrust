function workload(n: number, seed: number, input: string): string {
  const value = JSON.parse(input);
  if (value.tag !== 42) throw Error('unexpected tag');
  // Fixed benchmark envelope; this is not a general raw-field parser.
  const end = input.indexOf(',"tag":42,');
  if (!input.startsWith('{"payload":') || end < 0) throw Error('envelope mismatch');
  return input.slice(11, end);
}
