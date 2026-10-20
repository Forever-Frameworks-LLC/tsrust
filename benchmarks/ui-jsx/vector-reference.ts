// Independent SVG viewport and chart oracle; no renderer/source helpers.
declare const console:{log(value:string):void};
function viewport(policy:string,width:number,height:number){
 const sx=width/40,sy=height/20;
 if(policy==='none')return [0,0,width,height];
 const scale=policy.endsWith('slice')?Math.max(sx,sy):Math.min(sx,sy);
 const align=policy.startsWith('xMin')?[0,0]:policy.startsWith('xMax')?[1,1]:[.5,.5];
 const x=(width-40*scale)*align[0],y=(height-20*scale)*align[1];
 return [x,y,x+40*scale,y+20*scale];
}
function series(values:number[]){const line=values.map((value,index)=>(index?'L':'M')+(20+index*72)+' '+(180-value)).join('');return {line,area:'M20 180L'+line.slice(1)+'L380 180Z',text:'Builds by day: '+values.join(', ')+'.'};}
const policies=['xMidYMid meet','xMinYMin meet','xMaxYMax meet','xMidYMid slice','none'];
console.log(JSON.stringify({native:policies.map(policy=>({policy,width:200,height:200,corners:viewport(policy,200,200)})),browser:[['meet','xMidYMid meet'],['slice','xMidYMid slice'],['none','none'],['corner','xMaxYMax meet']].map(([id,policy])=>({id,policy,corners:viewport(policy,96,96)})),steps:[series([20,50,35,80,65,100]),series([50,85,70,120,95,155]),series([20,50,35,80,65,100])]}));
