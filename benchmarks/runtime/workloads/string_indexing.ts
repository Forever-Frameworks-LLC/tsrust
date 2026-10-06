type int=number;
function at(text:string,position:int):string{const found=text[position];if(found===undefined)throw new RangeError("string index out of range");return found;}
function workload(n: int, seed: int, input: string): string[][] {
 const output:string[][]=[];let i=0;const letters="abcdefghij";
 while(i<n){
  const k=i+seed;const position=k%10;
  const first=at(letters,position);
  const pair=at(letters,position)+at(letters,9-position);
  let later="";
  try{later=at(letters,position+5);}catch(error){later="out";}
  output.push([first,pair,later,String(position),String(input)]);i+=1;
 }
 return output;
}
function main(){const result=workload(3,0,"abc");}
