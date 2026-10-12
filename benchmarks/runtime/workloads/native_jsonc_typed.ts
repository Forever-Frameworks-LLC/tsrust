function stripComments(input: string): string {
  const output = input.split('');
  let quoted = false;
  for (let i = 0; i < input.length; i++) {
    if (quoted) {
      if (input[i] === '\\') { i++; continue; }
      if (input[i] === '"') quoted = false;
      continue;
    }
    if (input[i] === '"') { quoted = true; continue; }
    if (input[i] === '/' && input[i + 1] === '/') {
      while (i < input.length && input[i] !== '\n' && input[i] !== '\r') output[i++] = ' ';
      i--;
    } else if (input[i] === '/' && input[i + 1] === '*') {
      output[i++] = ' '; output[i++] = ' ';
      while (i < input.length && !(input[i] === '*' && input[i + 1] === '/')) {
        if (input[i] !== '\n' && input[i] !== '\r') output[i] = ' ';
        i++;
      }
      if (i >= input.length) throw Error('unterminated-comment');
      output[i++] = ' '; output[i] = ' ';
    }
  }
  return output.join('');
}
type Item = { name: string; values: number[]; active: boolean; };
function decodeItem(input: string): Item {
  const item: unknown = JSON.parse(stripComments(input));
  if (item === null || typeof item !== 'object' || Array.isArray(item)) throw Error('shape-mismatch');
  const fields = item as Record<string, unknown>;
  if (typeof fields.name !== 'string' || typeof fields.active !== 'boolean' || !Array.isArray(fields.values)) throw Error('shape-mismatch');
  for (const value of fields.values) {
    if (typeof value !== 'number' || !Number.isSafeInteger(value)) throw Error('integer-range');
  }
  return fields as Item;
}
function workload(n: number, seed: number, input: string): number {
  let checksum = seed;
  let i = 0;
  while (i < n) {
    const item = decodeItem(input);
    if (!item.active) { throw 'Inactive item'; }
    checksum += item.name.length + item.values[0]! + item.values[1]!;
    i++;
  }
  return checksum;
}
