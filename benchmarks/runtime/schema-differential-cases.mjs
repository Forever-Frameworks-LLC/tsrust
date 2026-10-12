// Exact reference adapters for the currently implemented, comparable S11 surface.
import {z} from 'zod';
export const schemaCases=[
 {id:'string-codepoints',native:'s.string().min(2).max(4)',reference:()=>z.string().refine(value=>[...value].length>=2&&[...value].length<=4),inputs:['"ok"','"x"','"hello"','"é😀"','"é"','42','null']},
 {id:'integer-bounds',native:'s.int().gt(-3).lt(4)',reference:()=>z.int().gt(-3).lt(4),inputs:['-3','-2','0','3','4','1.5','"2"','true']},
 {id:'number-bounds',native:'s.number().min(-1.5).max(2.5)',reference:()=>z.number().min(-1.5).max(2.5),inputs:['-2','-1.5','-0.0','2','2.5','3','"2"']},
 {id:'boolean',native:'s.boolean()',reference:()=>z.boolean(),inputs:['true','false','0','"true"','null']},
 {id:'null',native:'s.null()',reference:()=>z.null(),inputs:['null','false','0','[]','{}']},
 {id:'nested-array',native:'s.array(s.array(s.int().min(0)))',reference:()=>z.array(z.array(z.int().min(0))),inputs:['[]','[[1,2],[]]','[[1,-2],["wrong"]]','[null]','{}']},
 {id:'strict-object',native:'s.object({name:s.string(),count:s.int().min(1)})',reference:()=>z.strictObject({name:z.string(),count:z.int().min(1)}),inputs:['{"name":"ok","count":2}','{}','{"name":3,"count":0}','{"name":"ok","count":1,"extra":true}','null']},
 {id:'strip-object',native:'s.object({count:s.int()}).strip()',reference:()=>z.object({count:z.int()}),inputs:['{"count":2,"extra":[1,2]}','{"extra":true}','{"count":"x"}','{"count":1,"count":3}']},
 {id:'presence',native:'s.object({name:s.string().optional(),count:s.int().nullable()})',reference:()=>z.strictObject({name:z.string().optional(),count:z.int().nullable()}),inputs:['{"count":null}','{"name":"ok","count":2}','{}','{"name":null,"count":2}']},
 {id:'validated-default',native:"s.object({name:s.string().min(2).default(()=>'ready')})",reference:()=>z.strictObject({name:z.string().min(2).prefault(()=>'ready')}),inputs:['{}','{"name":"yes"}','{"name":"x"}','{"name":null}']},
 {id:'invalid-default',native:"s.object({name:s.string().min(2).default(()=>'x')})",reference:()=>z.strictObject({name:z.string().min(2).prefault(()=>'x')}),inputs:['{}','{"name":"yes"}','{"name":null}']},
 {id:'literal',native:"s.literal('ready')",reference:()=>z.literal('ready'),inputs:['"ready"','"done"','null','1']},
 {id:'enum',native:"s.enum(['ready','done'])",reference:()=>z.enum(['ready','done']),inputs:['"ready"','"done"','"wrong"','null']},
 {id:'ordered-union',native:'s.union([s.string().min(3),s.int().min(2)])',reference:()=>z.union([z.string().min(3),z.int().min(2)]),inputs:['"ready"','2','"x"','1','false','{}']},
 {id:'discriminated-union',native:"s.discriminatedUnion('kind',[s.object({kind:s.literal('text'),text:s.string()}),s.object({kind:s.literal('count'),count:s.int()})])",reference:()=>z.discriminatedUnion('kind',[z.strictObject({kind:z.literal('text'),text:z.string()}),z.strictObject({kind:z.literal('count'),count:z.int()})]),inputs:['{"kind":"text","text":"yes"}','{"kind":"count","count":3}','{"kind":"count","count":"bad"}','{"kind":"other"}','{}']},
 {id:'readonly-refinement',native:'s.int().refine(value=>value>2)',reference:()=>z.int().refine(value=>value>2),inputs:['3','2','"3"']},
 {id:'owned-transform',native:'s.array(s.int()).transform(values=>{values.push(9);return values.length;})',reference:()=>z.array(z.int()).transform(values=>{values.push(9);return values.length;}),inputs:['[]','[1,2]','["bad"]','null']},
 {id:'owned-coercion',modes:['fused'],native:'s.int().min(3).coerce((values:int[]):int=>{values.push(9);return values.length;})',reference:()=>z.array(z.int()).transform(values=>{values.push(9);return values.length;}).pipe(z.int().min(3)),inputs:['[1,2]','[]','[1]','["bad"]','null']},
];

export const differences=[
 'Object rejection uses z.strictObject; Tsrust rejects unknown keys by default.',
 'String lengths use Unicode code points; the reference uses an explicit code-point refinement instead of Zod UTF-16 length bounds.',
 'Tsrust validates factory defaults; the reference uses Zod prefault rather than default.',
 'Tsrust consumes owned input. Zod clones validated output. This check compares observable results, not allocation or aliasing.',
 'Output comparisons normalize both sides through JavaScript JSON serialization; native negative zero is retained while JavaScript JSON.stringify writes zero.',
 'Integer comparisons stay within the shared exact JavaScript integer domain. Native signed 64-bit boundaries have separate compiler/runtime evidence.',
 'Failure comparison covers rejection and property/index locations, not identical error codes, messages, ordering or branch issue multiplicity.',
 'Consuming-transform union commitment intentionally differs; fallback after consumption is covered by native contract witnesses, not asserted equivalent.',
 'Typed coercion validates its raw data shape before the callback; the reference models that with an explicit input schema, transform and pipe.',
];

export function referencePaths(issues){
 const paths=[];
 for(const issue of issues){
  if(issue.code==='invalid_union'&&issue.errors?.length){for(const errors of issue.errors)paths.push(...referencePaths(errors));}
  else if(issue.code==='unrecognized_keys'){for(const key of issue.keys)paths.push([...issue.path,key]);}
  else paths.push(issue.path);
 }
 return paths;
}
export function normalizedPaths(paths){return [...new Set(paths.map(path=>JSON.stringify(path)))].sort();}
export function nativePaths(issues){return issues.map(issue=>issue.path.map(segment=>segment.field??segment.index));}

export function scenarioModes(scenario){return scenario.modes??['owned','fused'];}
export function nativeSource(){
 const functions=schemaCases.map((scenario,index)=>{
  const owned=scenarioModes(scenario).includes('owned')?`
  match(schema.safeParse(JSON.parse(text))){
   Ok(value)=>{console.log(${JSON.stringify(scenario.id)},index,'owned','ok',JSON.stringify(value));},
   Err(error)=>{console.log(${JSON.stringify(scenario.id)},index,'owned','err',JSON.stringify(error.issues));}
  }`:'';
  return `
function scenario${index}(){
 const schema=${scenario.native};
 const inputs=[${scenario.inputs.map(text=>JSON.stringify(text)).join(',')}];
 let index=0;
 for(const text of inputs){
  ${owned}
  match(schema.safeParseJson(text)){
   Ok(value)=>{console.log(${JSON.stringify(scenario.id)},index,'fused','ok',JSON.stringify(value));},
   Err(failure)=>{if(failure.kind==='validation'){console.log(${JSON.stringify(scenario.id)},index,'fused','err',JSON.stringify(failure.error.issues));}else{throw 'Unexpected JSON failure';}}
  }
  index++;
 }
}`;}).join('\n');
 return "import {s} from 'std:schema';\n"+functions+'\nfunction main(){'+schemaCases.map((_,index)=>`scenario${index}();`).join('')+'}\n';
}
