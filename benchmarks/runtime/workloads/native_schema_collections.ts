// Handwritten ownership-independent reference; not a Zod allocation comparison.
type Outcome<T>={ok:true,value:T}|{ok:false,code:string};
function validateMap(input:Map<string,number>,lower=false):Outcome<Map<string,number>>{
 const output=new Map<string,number>();
 for(const [key,value] of input){
  if(typeof key!=='string'||!Number.isSafeInteger(value))return {ok:false,code:'type-mismatch'};
  const result=lower?key.toLowerCase():key;
  if(output.has(result))return {ok:false,code:'collection-collision'};
  output.set(result,value);
 }
 return {ok:true,value:output};
}
function validateSet(input:Set<string>,lower=false):Outcome<Set<string>>{
 const output=new Set<string>();
 for(const value of input){
  if(typeof value!=='string')return {ok:false,code:'type-mismatch'};
  const result=lower?value.toLowerCase():value;
  if(output.has(result))return {ok:false,code:'collection-collision'};
  output.add(result);
 }
 return {ok:true,value:output};
}
function workload(n:number,seed:number,input:string):number{
 let checksum=seed;let i=0;
 while(i<n){
  const data:unknown=JSON.parse(input);
  if(data===null||typeof data!=='object'||Array.isArray(data))throw Error('Record rejected');
  const record=new Map<string,number>();
  for(const [key,value] of Object.entries(data)){if(typeof value!=='number'||!Number.isSafeInteger(value))throw Error('Record value rejected');record.set(key,value);}
  if(record.size!==1||record.get('x')!==2)throw Error('Wrong record');checksum+=2;
  const map=validateMap(new Map([['x',3]]));
  if(!map.ok||map.value.size!==1||map.value.get('x')!==3)throw Error('Wrong map');checksum+=3;
  const set=validateSet(new Set(['ready']));
  if(!set.ok||set.value.size!==1||!set.value.has('ready'))throw Error('Wrong set');checksum+=1;
  const collisionMap=validateMap(new Map([['A',1],['a',2]]),true);
  if(collisionMap.ok||collisionMap.code!=='collection-collision')throw Error('Wrong collision');checksum+=1;
  const collisionSet=validateSet(new Set(['A','a']),true);
  if(collisionSet.ok||collisionSet.code!=='collection-collision')throw Error('Wrong collision');checksum+=1;
  i++;
 }
 return checksum;
}
