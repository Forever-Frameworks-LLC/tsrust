function nestedSchema() {
 return {safeParseJson(text:string):{groups:{values:number[]}[]} {
  const input:unknown=JSON.parse(text);
  if(typeof input!=='object'||input===null||Array.isArray(input)||Object.keys(input).join(',')!=='groups')throw 'Wrong object';
  const groups=(input as {groups:unknown}).groups;
  if(!Array.isArray(groups))throw 'Wrong array';
  return {groups:groups.map(group=>{
   if(typeof group!=='object'||group===null||Array.isArray(group)||Object.keys(group).join(',')!=='values'||!Array.isArray(group.values))throw 'Wrong group';
   return {values:group.values.map((value:unknown)=>{if(typeof value!=='number'||!Number.isInteger(value)||value<0||value>1000)throw 'Wrong integer';return value;})};
  })};
 }};
}
function workload(n:number,seed:number,input:string):number {
 const schema=nestedSchema();let checksum=seed;let i=0;
 while(i<n){
  const value=schema.safeParseJson(input);
  if(value.groups.length!==4)throw 'Wrong groups';
  for(const group of value.groups){
   if(group.values.length!==16)throw 'Wrong values';
   for(const item of group.values){checksum+=item;}
  }
  i++;
 }
 return checksum;
}
