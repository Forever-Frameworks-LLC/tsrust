// Independent manual visitation/arithmetic/segment-copy expectations.
// Never invoke the source declared functions, map, reduce, replaceAll or clone.
const MOD=1000000007n;
export const rowTypes={declared_callback_int_reduce:'int',declared_callback_number_reduce:'number',declared_callback_boolean_reduce:'boolean',declared_callback_string_map:'string[][]',declared_callback_record_map:'Row[][]',declared_value_local_reuse:'int',declared_value_local_move:'int',declared_value_returned_function:'int',declared_value_factory_target:'string[]',declared_value_global_use:'int',declared_value_bound_capture:'int',declared_value_owned_tree_move:'Node[]'};
export const defaultInput=' xé😀e\u0301 ';
function textBound(input){if(typeof input!=='string')throw Error('Text input required');let bytes=0;for(const c of input){const n=c.codePointAt(0);if(n>=0xd800&&n<=0xdfff)throw Error('Valid Unicode required');bytes+=n<=127?1:n<=2047?2:n<=65535?3:4;}if(bytes>256)throw Error('Text bound256 bytes');}
function validate(id,n,seed,input){if(!Object.hasOwn(rowTypes,id))throw Error('Unknown declared value row');const text=rowTypes[id].includes('[]');if(!Number.isSafeInteger(n)||n<0||n>(text?512:2048)||!Number.isSafeInteger(seed)||seed<0||seed>1000000)throw Error('Bounded n/seed required');if(text)textBound(input);}
function count(i,seed){const mode=(i+seed)%4;return mode===0?0:mode===1?1:mode===2?4:5;}
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
function replaceX(value,prefix){return manualReplaceX(value,prefix);}
function copyItems(values){const result=[];for(let i=0;i<values.length;i++)result.push(values[i]);return result;}
export function expectedDeclared(id,n,seed,input=defaultInput){
 validate(id,n,seed,input);
 if(id==='declared_callback_number_reduce'){let quarters=0n;const base=[1n,-2n,6n,9n,-4n];for(let i=0;i<n;i++)for(let j=0;j<count(i,seed);j++)quarters+=base[j];return Number(quarters)/4;}
 if(id==='declared_callback_boolean_reduce'){const base=[true,false,seed%2===0,false,true];let output=false;for(let i=0;i<n;i++){let parity=false;for(let j=0;j<count(i,seed);j++)if(base[j])parity=!parity;if(parity)output=!output;}return output;}
 if(id==='declared_callback_string_map'){const base=[input,'x😀','a\u0000b','e\u0301','xxé'],output=[];for(let i=0;i<n;i++){const row=[];for(let j=0;j<count(i,seed);j++)row.push(replaceX(base[j],'é\u0000'));output.push(row);}return output;}
 if(id==='declared_callback_record_map'){const base=[{code:seed,text:input,items:[seed,1]},{code:2,text:'x😀',items:[]},{code:3,text:'a\u0000b',items:[2,3]},{code:4,text:'e\u0301',items:[4]},{code:5,text:'xxé',items:[5,6]}],output=[];for(let i=0;i<n;i++){const values=[];for(let j=0;j<count(i,seed);j++)values.push({code:base[j].code+j+1,text:base[j].text,items:copyItems(base[j].items)});output.push(values);}return output;}
 if(id==='declared_value_factory_target'){const prefixes=[input,'a\u0000b','😀','e\u0301'],output=[];for(let i=0;i<n;i++)output.push(replaceX(input,prefixes[(seed+i)%4]));return output;}
 if(id==='declared_value_owned_tree_move'){const output=[];for(let i=0;i<n;i++)output.push({code:seed+i,text:input,children:[{code:i,text:'a\u0000b',children:[]},{code:i+1,text:'e\u0301😀',children:[{code:seed,text:'leaf',children:[]}]}]});return output;}
 let trace=BigInt(seed);
 for(let i=0;i<n;i++){
  if(id==='declared_callback_int_reduce'){const base=[seed+1,2,seed+3,4,seed+5];let value=0n;for(let j=0;j<count(i,seed);j++)value=(value*17n+BigInt(base[j]))%MOD;trace=(trace*131n+value)%MOD;continue;}
  const first=(3n*(BigInt(seed)+BigInt(i))+1n)%MOD;
  if(id==='declared_value_local_reuse'){const second=(3n*first+1n)%MOD;trace=(trace*131n+first+second)%MOD;}
  else if(id==='declared_value_local_move'){const second=(3n*(BigInt(seed)+BigInt(i)+1n)+1n)%MOD;trace=(trace*131n+first+second)%MOD;}
  else if(id==='declared_value_returned_function'){const state=((BigInt(seed)+BigInt(i))%1000003n*10n+7n)%1000003n;trace=(trace*131n+first+state)%MOD;}
  else trace=(trace*131n+first)%MOD;
 }
 return Number(trace);
}
// Exact quarter rational IEEE encoding, independent of DataView/native f64.
function quarterBits(quarters){const negative=quarters<0n;if(negative)quarters=-quarters;const sign=negative?1n<<63n:0n;if(quarters===0n)return sign;const exponent=quarters.toString(2).length-3;const mantissa=quarters<<BigInt(50-exponent);return sign|(BigInt(exponent+1023)<<52n)|(mantissa-(1n<<52n));}
export function expectedDeclaredObservation(id,n,seed,input){const value=expectedDeclared(id,n,seed,input),type=rowTypes[id];if(type==='int')return{checksum:value};if(type==='boolean')return{checksum:value?1:0,exactBits:value?'b:1':'b:0'};
 let checksum=0n;const add=n=>{checksum=(checksum*131n+BigInt(n))%MOD;};
 if(type==='number'){const bits=quarterBits(BigInt(value*4));for(let shift=56n;shift>=0n;shift-=8n)add((bits>>shift)&255n);return{checksum:Number(checksum),exactBits:'n:'+bits.toString(16).padStart(16,'0')};}
 function visit(value,type){if(type.endsWith('[]')){add(256);const pieces=[];for(let i=0;i<value.length;i++)pieces.push(visit(value[i],type.slice(0,-2)));add(257);return'['+pieces.join(',')+']';}
  if(type==='Row'||type==='Node'){add(256);const pieces=[visit(value.code,'int'),visit(value.text,'string'),visit(type==='Row'?value.items:value.children,type==='Row'?'int[]':'Node[]')];add(257);return'['+pieces.join(',')+']';}
  if(type==='int'){add(value);return String(value);}add(258);const pieces=[];for(const scalar of value){const n=scalar.codePointAt(0);add(n);pieces.push(n);}add(259);return's('+pieces.join(',')+')';}
 const shape=visit(value,type);return{checksum:Number(checksum),exactBits:'t:'+shape};
}
export const handExpected=[
 {id:'declared_callback_int_reduce',n:1,seed:1,value:133},
 {id:'declared_callback_number_reduce',n:2,seed:1,value:3.75},
 {id:'declared_callback_boolean_reduce',n:2,seed:1,value:false},
 {id:'declared_callback_string_map',n:1,seed:1,input:'xé',value:[['é\u0000é']]},
 {id:'declared_callback_record_map',n:1,seed:1,input:'x',value:[[{code:2,text:'x',items:[1,1]}]]},
 {id:'declared_value_local_reuse',n:1,seed:1,value:148},
 {id:'declared_value_local_move',n:1,seed:1,value:142},
 {id:'declared_value_returned_function',n:1,seed:1,value:152},
 {id:'declared_value_factory_target',n:1,seed:0,input:'xé',value:['xéé']},
 {id:'declared_value_global_use',n:2,seed:0,value:135},
 {id:'declared_value_bound_capture',n:2,seed:0,value:135},
 {id:'declared_value_owned_tree_move',n:1,seed:0,input:'x',value:[{code:0,text:'x',children:[{code:0,text:'a\u0000b',children:[]},{code:1,text:'e\u0301😀',children:[{code:0,text:'leaf',children:[]}]}]}]}
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
