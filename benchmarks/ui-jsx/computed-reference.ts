// Independent application transitions; no renderer, compiler or UI helpers.
declare const console:{log(value:string):void};
function snapshot(count:number,name:string,visible:boolean){return {arithmetic:`${count+1} / Item ${count}`,name:`Hello ${name.toUpperCase()}`,optional:`A${visible?'shown':''}B`,exact:'9007199254740993 / 1.25 / 0',flow:`Next: ${count*10}`,button:`Update ${count}`};}
console.log(JSON.stringify({observations:[snapshot(1,'Ada',false),snapshot(2,'Grace',true),snapshot(3,'Grace',false)],error:'Computed text failed',healthy:['11','12']}));
