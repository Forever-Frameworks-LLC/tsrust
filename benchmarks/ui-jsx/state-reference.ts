declare const console: {log(value:string):void};
// Independent application transitions: no signal, renderer or ownership implementation.
const states=Array.from({length:2},()=>({name:'Before',ready:false,amount:1.25,count:9007199254740993n}));
const observations: {text:string[],disabled:boolean[]}[]=[];
function observe(){observations.push({text:states.map(s=>`${s.name} / ${s.ready} / ${s.amount} / ${s.count}`),disabled:states.map(s=>!s.ready)});}
function change(index:number){const state=states[index];state.name+='!';state.ready=!state.ready;state.amount+=0.5;state.count+=1n;}
observe();change(0);observe();if(states[0].ready)states[0].name='Reset';observe();change(1);observe();change(1);observe();
const numbers=[-0,NaN,Infinity,-Infinity,1e21].map(String).join(' / ');
console.log(JSON.stringify({observations,numbers}));
