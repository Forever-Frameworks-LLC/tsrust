type Issue = {code:string;expected:string;observed:string;path:unknown[];pathTruncated:boolean};
type Validation = {ok:true;value:number}|{ok:false;error:{issues:Issue[];truncated:boolean}};
function pipelineSchema() {
  const coerce = (values:number[]):number => { values.push(7); return values.length; };
  const refine = (value:number):boolean => value < 10;
  const transform = (value:number):number => value + 1;
  return {safeParse(input:number[]):Validation {
    const value = coerce(input);
    const code = value < 3 ? 'too-small' : !refine(value) ? 'refinement-failed' : undefined;
    if (code !== undefined) return {ok:false,error:{issues:[{code,expected:'integer',observed:'integer',path:[],pathTruncated:false}],truncated:false}};
    return {ok:true,value:transform(value)};
  }};
}
function retrySchema() {
  const branches = [(values:number[]) => values.length > 3, (values:number[]) => true];
  return {safeParse(input:unknown):Validation {
    const issues:Issue[]=[];
    for (const [index, accept] of branches.entries()) {
      if (!Array.isArray(input) || !input.every(value=>Number.isSafeInteger(value))) {
        throw 'Benchmark input must be an integer array';
      }
      if (accept(input)) { issues.length=0; return {ok:true,value:index===0?0:input.length}; }
      issues.push({code:'refinement-failed',expected:'array',observed:'array',path:[],pathTruncated:false});
    }
    return {ok:false,error:{issues,truncated:false}};
  }};
}
function workload(n:number,seed:number,input:string):number {
  const schema = pipelineSchema();
  const retry = retrySchema();
  let checksum = seed;
  let i = 0;
  while (i < n) {
    const success = schema.safeParse([2, 3]);
    if (!success.ok) throw 'Expected success';
    const value = success.value;
    if (value !== 4) { throw 'Wrong transformed value'; }
    checksum += value;
    const failure = schema.safeParse([]);
    if (failure.ok) throw 'Expected rejection';
    const error = failure.error;
    if (error.truncated || error.issues.length !== 1) { throw 'Wrong issue count'; }
    for (const issue of error.issues) {
      if (issue.code !== 'too-small' || issue.expected !== 'integer' || issue.observed !== 'integer' || issue.pathTruncated || issue.path.length !== 0) {
        throw 'Wrong issue';
      }
      checksum += 1;
    }
    const restored = retry.safeParse(JSON.parse('[2,3]'));
    if (!restored.ok) throw 'Expected rollback success';
    if (restored.value !== 2) { throw 'Wrong rollback'; }
    checksum += restored.value;
    i++;
  }
  return checksum;
}
