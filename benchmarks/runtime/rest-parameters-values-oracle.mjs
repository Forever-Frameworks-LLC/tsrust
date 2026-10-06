export const ids = ['int','number','boolean','string','nested','record','null','undefined'].map(family=>'rest_parameter_'+family+'_pack').concat(['rest_parameter_default_prefix','rest_parameter_read_callback','rest_parameter_mutate_callback','rest_parameter_owned_once_callback','rest_parameter_owned_callable_pack']);
function text(input,value){let result='';for(let j=0;j<value%3+1;j++)result+=input;return result+':'+value;}
function item(family,value,input){if(family==='int')return value;if(family==='number')return [0,-0,5e-324,-2.5,.5,Infinity,-Infinity,NaN][value%8];if(family==='boolean')return value%2===0;if(family==='string')return text(input,value);if(family==='nested')return value%3===0?[]:[value,value+1];if(family==='record')return value+':'+text(input,value);if(family==='null')return 'object:null';if(family==='undefined')return 'undefined:undefined';throw Error(family);}
export function expectedValues(id,n,seed,input){const output=[];let state=seed%11;for(let i=0;i<n;i++){const mode=(i+seed)%3,base=(i*13+seed)%97,count=mode===0?0:mode===1?1:5;const values=[];for(let at=0;at<count;at++)values.push(base+at);
 if(id==='rest_parameter_default_prefix'){output.push([mode===2?base:10,...values],[count,mode===2?0:1]);}
 else if(id==='rest_parameter_read_callback'){const offset=seed%11;output.push([offset,...values.map(v=>v+offset)],[1]);}
 else if(id==='rest_parameter_mutate_callback'){state++;output.push([state,...values.map(v=>v+state)],[1]);}
 else if(id==='rest_parameter_owned_once_callback'){output.push([text(input,base),...values.map(v=>''+v)],['1']);}
 else if(id==='rest_parameter_owned_callable_pack'){output.push(values.map((_,at)=>base+(at%2===0?1:2)));}
 else {const family=id.slice('rest_parameter_'.length,-'_pack'.length);output.push(values.map(v=>item(family,v,input)));}
 }return output;}
// These literal expected cells are independent of expectedValues and all tested rest functions.
const literal={
 int:[[],[1],[2,3,4,5,6]],number:[[],[-0],[5e-324,-2.5,.5,Infinity,-Infinity]],boolean:[[],[false],[true,false,true,false,true]],
 string:[[],['xx:1'],['xxx:2','x:3','xx:4','xxx:5','x:6']],nested:[[],[[1,2]],[[2,3],[],[4,5],[5,6],[]]],
 record:[[],['1:xx:1'],['2:xxx:2','3:x:3','4:xx:4','5:xxx:5','6:x:6']],null:[[],['object:null'],['object:null','object:null','object:null','object:null','object:null']],undefined:[[],['undefined:undefined'],['undefined:undefined','undefined:undefined','undefined:undefined','undefined:undefined','undefined:undefined']],
 default_prefix:[[[10],[0,1]],[[10,1],[1,1]],[[2,2,3,4,5,6],[5,0]]],read_callback:[[[0],[1]],[[1,2],[1]],[[2,4,5,6,7,8],[1]]],mutate_callback:[[[1],[1]],[[2,3],[1]],[[3,5,6,7,8,9],[1]]],owned_once_callback:[[['x:0'],['1']],[['xx:1','1'],['1']],[['xxx:2','2','3','4','5','6'],['1']]],owned_callable_pack:[[],[2],[3,4,3,4,3]]};
export const hands=[];for(const [family,cells] of Object.entries(literal)){const id='rest_parameter_'+family+(['default_prefix','read_callback','mutate_callback','owned_once_callback','owned_callable_pack'].includes(family)?'':'_pack');for(let seed=0;seed<3;seed++){const expected=['default_prefix','read_callback','mutate_callback','owned_once_callback'].includes(family)?cells[seed]:[cells[seed]];hands.push([id,1,seed,'x',expected]);}}
hands.push(['rest_parameter_number_pack',1,1,'x',[[-0]]],['rest_parameter_number_pack',1,7,'x',[[NaN]]],['rest_parameter_string_pack',1,2,'é🙂\0',[['é🙂\0é🙂\0é🙂\0:2','é🙂\0:3','é🙂\0é🙂\0:4','é🙂\0é🙂\0é🙂\0:5','é🙂\0:6']]]);
