// Handwritten matched immutable plans, not a Zod performance comparison.
function textPlan(min: number, max: number) {
  return { toJsonSchema() { return { $schema: 'https://json-schema.org/draft/2020-12/schema', maxLength: max, minLength: min, type: 'string' }; } };
}
function flagsPlan() {
  return { toJsonSchema() { return { $schema: 'https://json-schema.org/draft/2020-12/schema', items: { type: 'boolean' }, type: 'array' }; } };
}
function workload(n: number, seed: number, input: string): number {
  const text = textPlan(2, 8);
  const flags = flagsPlan();
  let checksum = seed;
  let i = 0;
  while (i < n) {
    const first = JSON.stringify(text.toJsonSchema());
    const second = JSON.stringify(flags.toJsonSchema());
    if (first !== input || second !== '{"$schema":"https://json-schema.org/draft/2020-12/schema","items":{"type":"boolean"},"type":"array"}') {
      throw 'Invalid exported document';
    }
    checksum += first.length + second.length;
    i++;
  }
  return checksum;
}
