export const ids=['nullable_callable_higher_order_factory','nullable_callable_absent_higher_order_factory'];
export function expected(id,n,seed,input){
 if(!ids.includes(id)||!Number.isInteger(n)||n<0||n>2048||!Number.isInteger(seed)||seed<0||seed>1000000||typeof input!=='string'||input.length>256||[...input].some(c=>c.codePointAt(0)>127))throw new Error('Outside shared bounded ASCII domain');
 const output=[];
 for(let i=0;i<n;i++){
  const tag=(i+seed)%3,available=id===ids[1]||tag===0,value=(i*13+seed+input.length)%97;
  const result=((value*3+seed%11+7)%1009+i%7)%1009;
  output.push(available?[result]:[],id===ids[1]?[3,1,1,1,1,1]:available?[1,1,1,1,1,0]:[1,0,0,0,0,0]);
 }
 return output;
}
