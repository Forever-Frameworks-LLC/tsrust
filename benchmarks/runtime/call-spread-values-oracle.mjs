export const ids=['fixed_literal_int','dynamic_int','multiple_mixed_int','math_number','boolean_pack','string_owned','nested_owned','record_owned','nullish_packs','borrowed_string_fixed','optional_default_counts','mutate_callback','owned_once_optional'].map(v=>'call_spread_'+v);
function items(base,count){const values=[];for(let at=0;at<count;at++)values.push(base+at);return values;}
function text(input,value){let s='';for(let at=0;at<value%3+1;at++)s+=input;return s+':'+value;}
function repeat(input,count){let s='';for(let at=0;at<count;at++)s+=input;return s;}
function max(values){let selected=-Infinity;for(const v of values){if(v!==v)return NaN;if(v>selected||(v===0&&selected===0&&!Object.is(v,-0)))selected=v;}return selected;}
export function expectedValues(id,n,seed,input){const output=[];let state=seed%5;for(let i=0;i<n;i++){const mode=(i+seed)%3,base=(i*13+seed)%97,count=mode===0?0:mode===1?1:5,values=items(base,count),present=(i+seed)%4!==0;
 if(id==='call_spread_fixed_literal_int')output.push([base,base+mode+1,base+2]);
 else if(id==='call_spread_dynamic_int')output.push(values);
 else if(id==='call_spread_multiple_mixed_int'){const result=[base],left=items(base+10,mode===0?0:mode===1?1:3),rm=(mode+1)%3,right=items(base+20,rm===0?0:rm===1?1:3);for(const v of left)result.push(v);result.push(base+7);for(const v of right)result.push(v);result.push(base+9);output.push(result);}
 else if(id==='call_spread_math_number'){const pool=[0,-0,5e-324,-2.5,.5,Infinity,-Infinity,NaN],source=[];for(let at=0;at<count;at++)source.push(pool[(base+at)%8]);output.push([max(source)],source);}
 else if(id==='call_spread_boolean_pack'){const result=[];for(const v of values)result.push(v%2===0);output.push(result);}
 else if(id==='call_spread_string_owned'){const result=[];for(const v of values)result.push(text(input,v));output.push(result);}
 else if(id==='call_spread_nested_owned'){const result=[];for(const v of values)result.push(v%3===0?[]:[v,v+1]);output.push(result);}
 else if(id==='call_spread_record_owned'){for(const v of values)output.push([''+v,text(input,v)]);output.push([''+count]);}
 else if(id==='call_spread_nullish_packs'){const nil=[],abs=[];for(let at=0;at<count;at++){nil.push('object:null');abs.push('undefined:undefined');}output.push(nil,abs);}
 else if(id==='call_spread_borrowed_string_fixed'){const left=repeat('ab',mode+1),right=repeat('c',base%3+1);output.push([''+(left.length+right.length)],[left,right]);}
 else if(id==='call_spread_optional_default_counts'){if(present)output.push(count===0?[10]:values,[1,count===0?1:0]);else output.push([],[0,0]);}
 else if(id==='call_spread_mutate_callback'){state++;const result=[state];for(const v of values)result.push(v);output.push(result,[1]);}
 else if(id==='call_spread_owned_once_optional'){if(present){const result=[repeat(input,mode+1)];for(const v of values)result.push(''+v);output.push(result,['1','1']);}else output.push(['missing'],['0','0']);}
 else throw Error(id);
 }return output;}
// Independently written literal complete workload cells; no tested calls or oracle recomputation.
const literal={
 fixed_literal_int:[[[0,1,2]],[[1,3,3]],[[2,5,4]]],
 dynamic_int:[[[]],[[1]],[[2,3,4,5,6]]],
 multiple_mixed_int:[[[0,7,20,9]],[[1,11,8,21,22,23,10]],[[2,12,13,14,9,11]]],
 math_number:[[[-Infinity],[]],[[-0],[-0]],[[Infinity],[5e-324,-2.5,.5,Infinity,-Infinity]]],
 boolean_pack:[[[]],[[false]],[[true,false,true,false,true]]],
 string_owned:[[[]],[['xx:1']],[['xxx:2','x:3','xx:4','xxx:5','x:6']]],
 nested_owned:[[[]],[[[1,2]]],[[[2,3],[],[4,5],[5,6],[]]]],
 record_owned:[[['0']],[['1','xx:1'],['1']],[['2','xxx:2'],['3','x:3'],['4','xx:4'],['5','xxx:5'],['6','x:6'],['5']]],
 nullish_packs:[[[],[]],[['object:null'],['undefined:undefined']],[['object:null','object:null','object:null','object:null','object:null'],['undefined:undefined','undefined:undefined','undefined:undefined','undefined:undefined','undefined:undefined']]],
 borrowed_string_fixed:[[['3'],['ab','c']],[['6'],['abab','cc']],[['9'],['ababab','ccc']]],
 optional_default_counts:[[[],[0,0]],[[1],[1,0]],[[2,3,4,5,6],[1,0]]],
 mutate_callback:[[[1],[1]],[[2,1],[1]],[[3,2,3,4,5,6],[1]]],
 owned_once_optional:[[['missing'],['0','0']],[['xx','1'],['1','1']],[['xxx','2','3','4','5','6'],['1','1']]]
};
export const hands=[];for(const [name,cells] of Object.entries(literal))for(let seed=0;seed<3;seed++)hands.push(['call_spread_'+name,1,seed,'x',cells[seed]]);
hands.push(['call_spread_math_number',1,7,'x',[[NaN],[NaN]]],['call_spread_optional_default_counts',1,3,'x',[[10],[1,1]]],['call_spread_owned_once_optional',1,3,'x',[['x'],['1','1']]],['call_spread_string_owned',1,1,'é🙂\0',[['é🙂\0é🙂\0:1']]]);
