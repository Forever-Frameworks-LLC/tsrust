type int=number;
function describe([,...[text,fallback="tail"]]:string[]):string{return text+":"+fallback;}
function workload(n: int, seed: int, input: string): string[][] {
 const output:string[][]=[];let i=0;
 while (i < n) {
  const base=(i*13+seed)%97;const words=[input.repeat(1),String(base),input.repeat(2)];
  const [first,...[second,...tail]]=words;tail.push("end");
  const [,...{length}]=words;
  const tuple:[int,string,int]=[base,input.repeat(1),base+1];const [head,...[text,amount]]=tuple;
  output.push([first.slice(),second.slice(),tail.join("|"),String(length),String(head),text,String(amount),describe([input.repeat(1),input.repeat(2)]),String(words.length)]);
  i=i+1;
 }return output;
}
function main(){const result=workload(3,0,"abc");}
