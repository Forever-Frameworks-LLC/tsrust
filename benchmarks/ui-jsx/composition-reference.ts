// Independent application state oracle. Optional handlers do not mutate application state.
declare const console: {log(value: string): void};
let count=0;
const state=()=>({count:`Count: ${count}`,disabled:count<2});
const observed=[state()];
count+=1;observed.push(state());count+=1;observed.push(state());
observed.push(state());observed.push(state());
console.log(JSON.stringify(observed));
