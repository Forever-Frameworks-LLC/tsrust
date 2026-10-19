// Independent initial-branch and application-state oracle; no UI helpers.
declare const console:{log(value:string):void};
const tags=['SPAN','BUTTON','SPAN','BUTTON'];
function snapshot(first:number,second:number){return {flow:`BeforeBetweenCount: ${first}AfterCount: ${second}`,tags,text:'ABC',buttons:[`Count: ${first}`,`Count: ${second}`,'Ready']};}
console.log(JSON.stringify([snapshot(1,1),snapshot(2,1),snapshot(2,2)]));
