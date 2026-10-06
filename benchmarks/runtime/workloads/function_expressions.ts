type int=number;
function apply(action:(value:int)=>int,value:int):int{return action(value);}
function workload(n: int, seed: int, input: string): string[][] {
 const output:string[][]=[];let i=0;
 while (i < n) {
  const base=(i*13+seed)%97;
  const scale=function (value:int):int{return value*3+base;};
  const tags=[String(base),"z","tag"];
  const sizes=tags.map(function (tag){return tag.length;});
  const words=[input.repeat(1),String(base),"z"];
  words.sort(function (left,right){return left<right?-1:1;});
  let total=0;
  sizes.forEach(function (size:int){total+=size;});
  const join=function (first:string="a",...rest:string[]):string{return first+String(rest.length);};
  output.push([String(apply(scale,i%5)),String(total),words[0].slice(),join(),join("b","c","d")]);
  i=i+1;
 }return output;
}
function main(){const result=workload(3,0,"abc");}
