// Independent transition oracle. No renderer or Tsrust implementation helpers.
declare const console:{log(value:string):void};
function snapshot(active:boolean,count:number){return {tone:active?'destructive':'default',surface:active?'bg-muted':'bg-background',text:`Count: ${count} / doubled: ${count*2}`,disabled:count>=2};}
console.log(JSON.stringify({steps:[snapshot(false,0),snapshot(true,0),snapshot(true,1),snapshot(false,1),snapshot(false,2)],exact:(9007199254740993n+1n).toString(),buttons:2}));
