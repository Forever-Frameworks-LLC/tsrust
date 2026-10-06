// Independent manual traversal/append/copy oracle. No concat/slice/map/filter/reduce/forEach.
const MOD=1000000007n;
const kinds=new Set(['int','number','boolean','string','record','nested_int','nested_number','nested_boolean','nested_string','nested_record','deep_int','recursive_record']);
function row(code,text,items){return {code,text,items};}
function node(code,text,children){return {code,text,children};}
function validInput(input){if(typeof input!=='string')throw Error('String input required');let bytes=0;for(const scalar of input){const code=scalar.codePointAt(0);if(code>=0xd800&&code<=0xdfff)throw Error('Valid Unicode input required');bytes+=code<=0x7f?1:code<=0x7ff?2:code<=0xffff?3:4;}if(bytes>256)throw Error('Input bound256 UTF8 bytes');}
function validate(n,seed){if(!Number.isSafeInteger(n)||n<0||n>1000||!Number.isSafeInteger(seed)||seed<0||seed>1000000)throw Error('Bounded concat n0..1000/seed0..1000000 required');}
function copy(value){if(Array.isArray(value)){const result=[];for(let i=0;i<value.length;i++)result.push(copy(value[i]));return result;}if(value!==null&&typeof value==='object'){if(Object.hasOwn(value,'items'))return row(value.code,value.text,copy(value.items));return node(value.code,value.text,copy(value.children));}return value;}
function appendRange(output,values,first,last){for(let i=first;i<last;i++)output.push(copy(values[i]));}
function baseValues(kind,seed,input){switch(kind){
case'int':return [seed,seed+1,2,0,seed+4];
case'number':return [seed/4,-0,Infinity,-Infinity,NaN];
case'boolean':return [true,false,seed%2===0,true,false];
case'string':return [input,'é','😀','a\u0000b','e\u0301'];
case'record':return [row(seed,input,[seed,1]),row(2,'é',[]),row(3,'a\u0000b',[2,3]),row(4,'😀',[4]),row(5,'e\u0301',[5,6])];
case'nested_int':return [[seed,1],[],[2,3],[4],[5,6,7]];
case'nested_number':return [[seed/4,-0],[],[Infinity,-Infinity],[NaN],[0.25,1.5]];
case'nested_boolean':return [[true,false],[],[false],[true],[true,true,false]];
case'nested_string':return [[input,'é'],[],['😀','a\u0000b'],['e\u0301'],['last']];
case'nested_record':return [[row(seed,input,[seed])],[],[row(2,'é',[])],[row(3,'a\u0000b',[1])],[row(4,'😀',[2,3])]];
case'deep_int':return [[[seed],[]],[],[[1,2]],[[3]],[[4],[5]]];
case'recursive_record':return [node(seed,input,[node(1,'é',[])]),node(2,'😀',[]),node(3,'a\u0000b',[node(4,'e\u0301',[])]),node(5,'x',[]),node(6,'z',[])];
default:throw Error('Unknown concat domain');}}
function scalars(kind,seed,iteration,input){switch(kind){
case'int':return [seed+iteration,iteration+1];
case'number':return [iteration/4,-0];
case'boolean':return [(iteration+seed)%2===0,false];
case'string':{let value='';for(let i=0;i<(iteration+seed)%2+1;i++)value+=input;return [value,'a\u0000b'];}
case'record':return [row(seed+iteration,input,[iteration,seed]),row(iteration,'a\u0000b',[])];
case'recursive_record':return [node(seed+iteration,input,[node(iteration,'e\u0301',[])]),node(iteration,'a\u0000b',[])];
default:return undefined;}}
export function expectedConcat(id,n,seed,input='é 😀 e\u0301'){
 validate(n,seed);
 if(id==='array_concat_ordered_trace')return expectedOrderedTrace(n,seed);
 const match=/^array_concat_(.+)_(zero|variadic)$/.exec(id);if(!match||!kinds.has(match[1]))throw Error('Unknown concat row');
 const kind=match[1],mode=match[2];if(['string','record','nested_string','nested_record','recursive_record'].includes(kind))validInput(input);
 const base=baseValues(kind,seed,input),output=[];
 for(let iteration=0;iteration<n;iteration++){
  const selection=(iteration+seed)%4,count=selection===0?0:selection===1?1:selection===2?4:5,result=[];
  appendRange(result,base,0,count);
  if(mode==='variadic'){
   const scalar=scalars(kind,seed,iteration,input);if(scalar)result.push(copy(scalar[0]));
   appendRange(result,base,1,selection%3+2);
   if(scalar)result.push(copy(scalar[1]));
   // Explicit empty full-rank argument contributes no elements.
   appendRange(result,base,0,selection%2);
  }
  output.push(result);
 }
 return output;
}
function expectedOrderedTrace(n,seed){const base=baseValues('int',seed);let trace=BigInt(seed),effect=BigInt(seed);for(let iteration=0;iteration<n;iteration++){const mode=(iteration+seed)%4,count=mode===0?0:mode===1?1:mode===2?4:5;effect=(effect*10n+1n)%1000003n;effect=(effect*10n+2n)%1000003n;effect=(effect*10n+3n)%1000003n;const values=[];appendRange(values,base,0,count);appendRange(values,base,1,mode%3+2);values.push(seed+iteration);for(let index=0;index<values.length;index++)trace=(trace*131n+BigInt(values[index]+index+iteration))%MOD;trace=(trace*131n+effect)%MOD;}return Number(trace);}
// Exact finite quarter bits derived from integer numerator, not DataView/libm.
function floatToken(value){if(Number.isNaN(value))return {text:'nan',bits:0x7ff8000000000000n};if(value===Infinity)return {text:'7ff0000000000000',bits:0x7ff0000000000000n};if(value===-Infinity)return {text:'fff0000000000000',bits:0xfff0000000000000n};const negative=value<0||Object.is(value,-0);const numerator=BigInt(Math.abs(value)*4);let bits;if(numerator===0n)bits=negative?1n<<63n:0n;else{const exponent=numerator.toString(2).length-3,shift=50-exponent,mantissa=shift>=0?numerator<<BigInt(shift):numerator>>BigInt(-shift);bits=(negative?1n<<63n:0n)|(BigInt(exponent+1023)<<52n)|(mantissa-(1n<<52n));}return {text:bits.toString(16).padStart(16,'0'),bits};}
function leafSchema(kind){switch(kind){case'int':return'int';case'number':return'number';case'boolean':return'boolean';case'string':return'string';case'record':return'Row';case'recursive_record':return'Node';case'deep_int':return{array:{array:'int'}};default:if(kind.startsWith('nested_'))return{array:leafSchema(kind.substring(7))};throw Error('Unknown shape');}}
export function expectedConcatObservation(id,n,seed,input){const value=expectedConcat(id,n,seed,input);if(id==='array_concat_ordered_trace')return {checksum:value};const kind=/^array_concat_(.+)_(zero|variadic)$/.exec(id)[1],floating=kind==='number'||kind==='nested_number';let hash=0n;const add=value=>{hash=(hash*131n+BigInt(value))%MOD;};
 function visit(value,schema){if(typeof schema==='object'){if(!Array.isArray(value))throw Error('Expected array');add(256);const pieces=[];for(let i=0;i<value.length;i++)pieces.push(visit(value[i],schema.array));add(257);return'['+pieces.join(',')+']';}
 if(schema==='Row'||schema==='Node'){add(256);const pieces=[visit(value.code,'int'),visit(value.text,'string'),visit(schema==='Row'?value.items:value.children,{array:schema==='Row'?'int':'Node'})];add(257);return'['+pieces.join(',')+']';}
 if(schema==='number'){const token=floatToken(value);for(let shift=56n;shift>=0n;shift-=8n)add((token.bits>>shift)&255n);return token.text;}
 if(schema==='int'||schema==='boolean'){const integer=schema==='boolean'?(value?1:0):value;add(integer);return String(integer);}
 add(258);const codes=[];for(const scalar of value){const code=scalar.codePointAt(0);add(code);codes.push(code);}add(259);return's('+codes.join(',')+')';}
 const shape=visit(value,{array:{array:leafSchema(kind)}});return {checksum:Number(hash),exactBits:(floating?'m:':'t:')+shape};}
export const explicitExpected=[
 {id:'array_concat_int_zero',n:0,seed:7,value:[]},
 {id:'array_concat_int_zero',n:1,seed:0,value:[[]]},
 {id:'array_concat_int_zero',n:1,seed:1,value:[[1]]},
 {id:'array_concat_int_zero',n:1,seed:2,value:[[2,3,2,0]]},
 {id:'array_concat_int_variadic',n:1,seed:0,value:[[0,1,1]]},
 {id:'array_concat_int_variadic',n:1,seed:1,value:[[1,1,2,2,1,1]]},
 {id:'array_concat_number_variadic',n:1,seed:0,value:[[0,-0,-0]]},
 {id:'array_concat_nested_int_variadic',n:1,seed:0,value:[[[]]]},
 {id:'array_concat_string_variadic',n:1,seed:0,input:'é',value:[['é','é','a\u0000b']]},
 {id:'array_concat_ordered_trace',n:0,seed:7,value:7},
 {id:'array_concat_ordered_trace',n:1,seed:0,value:17415}
];
