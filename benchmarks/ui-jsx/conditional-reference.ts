// Application-state oracle; independent of either renderer and the compiler lowering.
declare const console:{log(value:string):void};
const observations:object[]=[];
let shown=true,second=false,count=1,name='stable',pulses=0;
function observe(label:string){observations.push({label,counter:shown?`${second?'Second':'First'}: ${count}`:null,status:`${name} / ${pulses}`,row:shown?['BUTTON','INPUT','BUTTON']:['INPUT','BUTTON']});}
observe('initial');count++;observe('increment');pulses++;observe('pulse');name='editing';observe('typing');shown=false;observe('hidden');shown=true;count=1;observe('shown during IME');second=true;count=100;observe('switched during IME');name='ed漢ing';observe('committed');count++;observe('second increment');second=false;count=1;observe('first restored');shown=false;pulses++;observe('hidden pulse');shown=true;count=1;observe('remounted');
console.log(JSON.stringify({observations,error:'Branch construction failed'}));
