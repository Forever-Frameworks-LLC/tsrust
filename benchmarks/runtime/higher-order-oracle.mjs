// SOURCE ONLY. Independent manual arithmetic, scalar substitution and traversal.
// Never invoke workload, its callbacks, clone, map, reduce or replaceAll.
export const defaultInput=' xé😀e\u0301 ';
const MOD=1000000007n;
export const rowTypes={
 higher_declared_reusable_int:'int',higher_inline_reusable_int:'int',higher_declared_once_int:'int',higher_inline_once_int:'int',higher_declared_owned_int:'int',higher_inline_owned_int:'int',higher_declared_owned_once_int:'int',higher_inline_owned_once_int:'int',higher_distinct_origins_int:'int',higher_multiple_callbacks_int:'int',higher_borrowed_declared_factory_int:'int',higher_borrowed_inline_factory_int:'int',higher_nested_consumer_int:'int',higher_owned_once_nested_int:'int',higher_number_reusable:'number',higher_boolean_reusable:'boolean',higher_owned_callback_return_string:'string[]',higher_captured_factory_string:'string[]',higher_read_string:'string[]',higher_consume_string:'string[]',higher_mutate_array:'int[][]',higher_owned_record:'Row[]',higher_deep_array:'int[][][]',higher_recursive_tree:'Node[]'
};
export function validateCase(id,n,seed,input=defaultInput){
 if(!Object.hasOwn(rowTypes,id))throw Error('Unknown higher-order row');
 const owned=rowTypes[id].endsWith('[]');
 if(!Number.isSafeInteger(n)||n<0||n>(owned?128:2048)||!Number.isSafeInteger(seed)||seed<0||seed>1000000)throw Error('Bounded n and seed required');
 if(typeof input!=='string')throw Error('String input required');let bytes=0;
 for(const scalar of input){const code=scalar.codePointAt(0);if(code>=0xd800&&code<=0xdfff)throw Error('Valid Unicode required');bytes+=code<=127?1:code<=2047?2:code<=65535?3:4;}
 if(bytes>256)throw Error('Input maximum 256 UTF8 bytes');
}
// Independent scalar scan and template interpreter; never call replace/replaceAll.
// Prefix/suffix refer to ORIGINAL source, and inserted text is never re-parsed.
export function manualReplaceX(value,template){
 const source=[],replacement=[];for(const scalar of value)source.push(scalar);for(const scalar of template)replacement.push(scalar);
 let output='';for(let position=0;position<source.length;position++){
  if(source[position]!=='x'){output+=source[position];continue;}
  for(let index=0;index<replacement.length;index++){
   const scalar=replacement[index];if(scalar!=='$'){output+=scalar;continue;}
   const next=replacement[index+1];
   if(next==='$'){output+='$';index++;}
   else if(next==='&'){output+='x';index++;}
   else if(next==='`'){for(let before=0;before<position;before++)output+=source[before];index++;}
   else if(next==="'"){for(let after=position+1;after<source.length;after++)output+=source[after];index++;}
   else output+='$'; // Unknown, dangling and capture-like tokens remain literal.
  }
 }return output;
}
function substituteX(value,prefix){return manualReplaceX(value,prefix);}
export function expectedHigher(id,n,seed,input=defaultInput){
 validateCase(id,n,seed,input);
 if(id==='higher_number_reusable'){let quarters=0n;for(let i=0;i<n;i++)quarters+=BigInt((seed+i)%17);return Number(quarters)/4;}
 if(id==='higher_boolean_reusable'){let parity=false;for(let i=0;i<n;i++)if((seed+i)%3!==0)parity=!parity;return parity;}
 if(rowTypes[id].endsWith('[]')){const output=[];for(let i=0;i<n;i++){
  if(id==='higher_owned_callback_return_string'){output.push(substituteX('x😀',input));output.push(substituteX('a\u0000bx',input));}
  else if(id==='higher_captured_factory_string')output.push(substituteX('xé',input));
  else if(id==='higher_read_string'||id==='higher_consume_string')output.push(input);
  else if(id==='higher_mutate_array'){output.push([seed+i+1,2,3]);output.push([seed+i+1,2,3]);}
  else if(id==='higher_owned_record')output.push({code:seed+i,text:input,items:[i,seed]});
  else if(id==='higher_deep_array')output.push([[seed+i,1],[],[i,2,3]]);
  else if(id==='higher_recursive_tree')output.push({code:seed+i,text:input,children:[{code:i,text:'a\u0000b',children:[]},{code:i+1,text:'😀',children:[{code:seed,text:'leaf',children:[]}]}]});
  else throw Error('Unmapped owned row');
 }return output;}
 let trace=BigInt(seed);
 for(let i=0;i<n;i++){
  const x=BigInt(seed)+BigInt(i);let value;
  if(id==='higher_distinct_origins_int'||id==='higher_nested_consumer_int')value=(x+3n)*7n+x*2n;
  else if(id==='higher_multiple_callbacks_int')value=(x+3n)*7n+x*2n+x*2n*7n+x+3n;
  else if(id==='higher_borrowed_declared_factory_int'||id==='higher_borrowed_inline_factory_int'){const a=BigInt(i)+BigInt(seed%11),b=BigInt(i)*BigInt(seed%7+1),c=BigInt(i+1)+BigInt(seed%11);value=a*7n+b+c;}
  else if(id==='higher_owned_once_nested_int')value=BigInt(i)+BigInt(seed%13);
  else value=(x*3n+1n)%MOD;
  trace=(trace*131n+value)%MOD;
 }
 return Number(trace);
}
// Exact positive quarter-rational encoding, independent of engine DataView/f64.
function quarterBits(quarters){if(quarters===0n)return 0n;const e=quarters.toString(2).length-3,m=quarters<<BigInt(50-e);return(BigInt(e+1023)<<52n)|(m-(1n<<52n));}
export function expectedHigherObservation(id,n,seed,input=defaultInput){
 const value=expectedHigher(id,n,seed,input),type=rowTypes[id];
 if(type==='int')return{checksum:value};if(type==='boolean')return{checksum:value?1:0,exactBits:value?'b:1':'b:0'};
 let sum=0n;const add=x=>{sum=(sum*131n+BigInt(x))%MOD;};
 if(type==='number'){const bits=quarterBits(BigInt(value*4));for(let shift=56n;shift>=0n;shift-=8n)add((bits>>shift)&255n);return{checksum:Number(sum),exactBits:'n:'+bits.toString(16).padStart(16,'0')};}
 function visit(v,t){if(t.endsWith('[]')){add(256);const tokens=[];for(let i=0;i<v.length;i++)tokens.push(visit(v[i],t.slice(0,-2)));add(257);return'['+tokens.join(',')+']';}
 if(t==='Row'||t==='Node'){add(256);const tokens=[visit(v.code,'int'),visit(v.text,'string'),visit(t==='Row'?v.items:v.children,t==='Row'?'int[]':'Node[]')];add(257);return'['+tokens.join(',')+']';}
 if(t==='int'){add(v);return String(v);}if(t!=='string')throw Error('Unmapped leaf');add(258);const tokens=[];for(const scalar of v){const code=scalar.codePointAt(0);add(code);tokens.push(code);}add(259);return's('+tokens.join(',')+')';}
 const shape=visit(value,type);return{checksum:Number(sum),exactBits:'t:'+shape};
}
// Small independently hand-derived cells; future checks must verify all.
export const handCells=[
 ['higher_declared_reusable_int',1,0,1],['higher_inline_owned_once_int',2,0,135],
 ['higher_distinct_origins_int',1,1,161],['higher_multiple_callbacks_int',1,0,24],
 ['higher_borrowed_declared_factory_int',1,1,140],['higher_owned_once_nested_int',2,1,17294],
 ['higher_number_reusable',3,0,0.75],['higher_boolean_reusable',3,0,false],
 ['higher_owned_callback_return_string',1,0,['xé😀','a\u0000bxé']],
 ['higher_captured_factory_string',1,0,['xéé']],['higher_deep_array',1,0,[[[0,1],[],[0,2,3]]]],
 ['higher_mutate_array',1,0,[[1,2,3],[1,2,3]]]
];

// Authored hand cells and future engine inputs; no checks executed.
export const replacementTemplateHandCells=[
 [
  "axbxc",
  "$$",
  "a$b$c"
 ],
 [
  "axbxc",
  "$&",
  "axbxc"
 ],
 [
  "axbxc",
  "$`",
  "aabaxbc"
 ],
 [
  "axbxc",
  "$'",
  "abxcbcc"
 ],
 [
  "axa",
  "[$$][$&][$`][$']",
  "a[$][x][a][a]a"
 ],
 [
  "xx",
  "$q",
  "$q$q"
 ],
 [
  "xx",
  "$",
  "$$"
 ],
 [
  "xx",
  "$1",
  "$1$1"
 ],
 [
  "xx",
  "$12",
  "$12$12"
 ],
 [
  "xx",
  "$$&",
  "$&$&"
 ],
 [
  "éx\u0000😀x",
  "$$",
  "é$\u0000😀$"
 ],
 [
  "éx\u0000😀x",
  "$&",
  "éx\u0000😀x"
 ],
 [
  "😀x\u0000xé",
  "$1$",
  "😀$1$\u0000$1$é"
 ],
 [
  "ax$&x",
  "$`",
  "aa$&ax$&"
 ],
 [
  "no😀\u0000match",
  "$$$&$`$'",
  "no😀\u0000match"
 ]
];
export const replacementTemplateInputs=["", "xé", "a\u0000b😀", "$$", "$&", "$`", "$'", "$q", "$", "$1", "xx$$", "x$&x", "x$`x", "x$'x", "😀x\u0000$$$&xé", "$12", "xx$q$1$", "😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀"];
