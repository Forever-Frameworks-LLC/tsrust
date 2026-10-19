type Item = { name: 'hello' | 'other'; values: number[]; active: true; };
function decodeItem(input: string): Item {
  const item: unknown = JSON.parse(input);
  if (item === null || typeof item !== 'object' || Array.isArray(item)) throw Error('shape-mismatch');
  const fields = item as Record<string, unknown>;
  if ((fields.name !== 'hello' && fields.name !== 'other') || fields.active !== true || !Array.isArray(fields.values)) throw Error('shape-mismatch');
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
    checksum += String(item.name).length + item.values[0]! + item.values[1]!;
    i++;
  }
  return checksum;
}
