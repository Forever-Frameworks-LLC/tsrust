// Independent interaction oracle; no component or style implementation imports.
declare const console:{log(value:string):void};
const snapshot=(destructive:boolean,neutral:boolean,count:number)=>({variant:neutral?null:destructive?'destructive':'default',padding:destructive?12:8,label:`Build ${count}`});
console.log(JSON.stringify({variants:['default','secondary','destructive','outline','ghost','link'],steps:[snapshot(false,false,0),snapshot(true,false,0),snapshot(true,false,1),snapshot(true,true,1),snapshot(false,true,1),snapshot(false,false,1)]}));
