type Tree = { value: number; children: Tree[] };
type Segment = { field?: string; index?: number };
type Issue = { code: string; pathTruncated: boolean; path: Segment[] };
type Validation = { ok: true; value: Tree } | { ok: false; error: { truncated: boolean; issues: Issue[] } };
// Matched recursive contract on this corpus; handwritten, not a Zod comparison.
function treeSchema() {
  function visit(input: unknown, path: Segment[], issues: Issue[]): boolean {
    const fail = (at: Segment[]) => { issues.push({ code: 'type-mismatch', pathTruncated: false, path: at }); return false; };
    if (input === null || typeof input !== 'object' || Array.isArray(input)) return fail(path);
    const value = input as Record<string, unknown>;
    let valid = true;
    if (typeof value.value !== 'number' || !Number.isSafeInteger(value.value)) valid = fail([...path, { field: 'value' }]);
    if (!Array.isArray(value.children)) valid = fail([...path, { field: 'children' }]);
    else for (let index = 0; index < value.children.length; index++) {
      if (!visit(value.children[index], [...path, { field: 'children' }, { index }], issues)) valid = false;
    }
    if (Object.keys(value).some(key => key !== 'value' && key !== 'children')) valid = fail(path);
    return valid;
  }
  return { safeParse(input: unknown): Validation {
    const issues: Issue[] = [];
    return visit(input, [], issues) ? { ok: true, value: input as Tree } : { ok: false, error: { truncated: false, issues } };
  } };
}
function workload(n: number, seed: number, input: string): number {
  const schema = treeSchema();
  let checksum = seed;
  let i = 0;
  while (i < n) {
    const success = schema.safeParse(JSON.parse(input));
    if (!success.ok) throw 'Expected tree success';
    const value = success.value;
    checksum += value.value + value.children[0].value;
    const failure = schema.safeParse(JSON.parse('{"value":1,"children":[{"value":"bad","children":[]}]}'));
    if (failure.ok) throw 'Expected nested rejection';
    const error = failure.error;
    if (error.truncated || error.issues.length !== 1) throw 'Invalid error count';
    for (const issue of error.issues) {
      if (issue.code !== 'type-mismatch' || issue.pathTruncated || issue.path.length !== 3 || issue.path[0].field !== 'children' || issue.path[1].index !== 0 || issue.path[2].field !== 'value') throw 'Invalid recursive path';
      checksum += 1;
    }
    i++;
  }
  return checksum;
}
