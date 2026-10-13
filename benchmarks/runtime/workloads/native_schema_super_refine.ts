type Issue = { code: string; message: string; path: unknown[]; pathTruncated: boolean; expected: string; observed: string };
type Validation = { ok: true; value: number } | { ok: false; error: { issues: Issue[]; truncated: boolean } };
// Handwritten matched contract on the fixed exact-integer corpus, not Zod.
function schemaPlan() {
  return { safeParse(value: unknown): Validation {
    if (typeof value !== 'number' || !Number.isSafeInteger(value)) throw 'Invalid corpus';
    const issues: Issue[] = [];
    const ctx = { addIssue({ message }: { message: string }) {
      issues.push({ code: 'refinement-failed', message, path: [], pathTruncated: false, expected: 'integer', observed: 'integer' });
    } };
    if (value < 1) { ctx.addIssue({ message: 'Must be positive' }); }
    if (value % 2 === 0) { ctx.addIssue({ message: 'Must be odd' }); }
    return issues.length === 0 ? { ok: true, value } : { ok: false, error: { issues, truncated: false } };
  } };
}
function workload(n: number, seed: number, input: string): number {
  const schema = schemaPlan();
  let checksum = seed;
  let i = 0;
  while (i < n) {
    const success = schema.safeParse(JSON.parse(input));
    if (!success.ok) throw 'Expected success';
    const value = success.value;
    checksum += value;
    const failure = schema.safeParse(JSON.parse('0'));
    if (failure.ok) throw 'Expected rejection';
    const error = failure.error;
    if (error.truncated || error.issues.length !== 2) { throw 'Invalid issue count'; }
    if (error.issues[0].message !== 'Must be positive' || error.issues[1].message !== 'Must be odd') { throw 'Invalid messages'; }
    for (const issue of error.issues) {
      if (issue.code !== 'refinement-failed' || issue.path.length !== 0 || issue.pathTruncated || issue.expected !== 'integer' || issue.observed !== 'integer') { throw 'Invalid issue'; }
      checksum += 1;
    }
    i++;
  }
  return checksum;
}
