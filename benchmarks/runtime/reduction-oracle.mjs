// Independent manual traversal: never calls reduce/reduceRight/slice/replace/repeat.
const M=1000000007n;
function copy(v){if(Array.isArray(v)){const a=[];for(const x of v)a.push(copy(x));return a;}if(v!==null&&typeof v==='object'){const a={};for(const k of Object.keys(v))a[k]=copy(v[k]);return a;}return v;}
function row(total,text,rows){return {total,text,rows,trace:0,visits:0};}
function firstMark(text,replacement){const needle='MARK';let at=-1;for(let i=0;i+4<=text.length;i++){let match=true;for(let k=0;k<4;k++)if(text[i+k]!==needle[k])match=false;if(match){at=i;break;}}if(at<0)return text;let s='';for(let i=0;i<at;i++)s+=text[i];s+=replacement;for(let i=at+4;i<text.length;i++)s+=text[i];return s;}
function setup(kind,seed){switch(kind){
case'int':case'int_to_record':case'int_to_array':case'int_to_nested':return [seed,seed+1,2,3,seed+4];
case'number':return [seed+.25,-0,2.5,-1.25,4.75];
case'boolean':return [true,false,seed%2===0,true,false];
case'string':return ['MARKa','MARKé','MARK😀','MARK\u0000','MARKz'];
case'record':return [row(seed,'é',[[seed]]),row(2,'😀',[]),row(3,'a\u0000b',[[1,2]]),row(4,'e\u0301',[[]]),row(5,'last',[[5]])];
case'nested_int':return [[seed,1],[],[2,3],[4],[5,6,7]];
case'nested_number':return [[seed+.25,-0],[],[2.5,-1.25],[4.75],[0]];
case'nested_boolean':return [[true,false],[],[false],[true],[true,true,false]];
case'nested_string':return [['é','😀'],[],['a\u0000b'],['e\u0301'],['last']];
case'nested_record':return [[row(seed,'é',[[seed]])],[],[row(2,'😀',[])],[row(3,'x',[[1]])],[]];
case'deep_int':return [[[seed],[]],[],[[1,2]],[[3]],[[4],[5]]];
default:throw Error('Unknown reduction domain');}}
function initial(kind){if(kind==='int')return 0n;if(kind==='number')return 0n; // exact integer quarters
if(kind==='boolean')return false;if(kind==='string')return 'MARK';if(kind==='record'||kind==='int_to_record')return row(0,'initial',[]);return [];}
export function expectedReduction(id,n,seed){const m=/^array_reduce(_right)?_(.+)$/.exec(id);if(!m||!Number.isSafeInteger(n)||n<0||n>2000||!Number.isSafeInteger(seed)||seed<0||seed>1000000)throw Error('Bounded reduction domain required');const reverse=Boolean(m[1]),kind=m[2];if(kind==='string'&&n>128)throw Error('Bounded owned string construction');const base=setup(kind,seed),out=[];let trace=BigInt(seed),visits=0n,last=initial(kind);const scalar=['int','number','boolean','string'].includes(kind),record=kind==='record'||kind==='int_to_record',noinitial=!kind.startsWith('int_to_');
for(let iteration=0;iteration<n;iteration++){const mode=(iteration+seed)%5,count=mode===0?0:mode===1?1:mode===2?4:5,values=[];for(let k=0;k<count;k++)values.push(copy(base[k]));let a=(scalar||kind==='record')?copy(last):initial(kind),k=reverse?count-1:0,step=reverse?-1:1;
if(noinitial&&mode===3){if(count===0)throw Error('No seed');a=copy(values[k]);if(kind==='int')a=BigInt(a);if(kind==='number')a=BigInt(a*4);k+=step;}
for(;k>=0&&k<count;k+=step){const v=values[k];trace=(trace*131n+BigInt(k+count+iteration))%M;visits++;
if(kind==='int')a=(a*17n+BigInt(v+k+count))%1000003n;
else if(kind==='number')a=(a*2n+BigInt(v*4+k+count))%4000012n;
else if(kind==='boolean')a=(a!==v)!==(k%2===0);
else if(kind==='string')a=firstMark(a,v);
else if(kind==='record')a={total:Number((BigInt(a.total)*17n+BigInt(v.total+k))%1000003n),text:v.text,rows:copy(v.rows),trace:Number(trace),visits:Number(visits)};
else if(kind==='int_to_record'){a.rows.push([v,k]);a={total:Number((BigInt(a.total)*17n+BigInt(v+k))%1000003n),text:a.text==='initial'?'owned':a.text,rows:a.rows,trace:Number(trace),visits:Number(visits)};}
else if(kind==='int_to_array')a.push(v+k);
else if(kind==='int_to_nested')a.push([v,k]);
else a=copy(v);
}
if(scalar||record)last=a;else out.push(copy(a));}
if(kind==='int')return Number((last+trace+visits)%M);if(kind==='number')return Number(last+trace+visits)/4;if(kind==='boolean')return last!==(trace%2n===0n);if(kind==='string'){let s='';for(let i=0;i<Number(trace%2n)+1;i++)s+=last;return s;}if(record)return last;return out;}
// Hand-calculated exact cells independent of the traversal implementation.
export const explicitExpected=[{id:'array_reduce_int',n:0,seed:7,value:7},{id:'array_reduce_right_int',n:0,seed:7,value:7},{id:'array_reduce_int',n:1,seed:7,value:70636345},{id:'array_reduce_right_int',n:1,seed:7,value:77391373},{id:'array_reduce_number',n:0,seed:7,value:1.75},{id:'array_reduce_boolean',n:0,seed:7,value:false},{id:'array_reduce_string',n:0,seed:7,value:'MARKMARK'},{id:'array_reduce_record',n:0,seed:7,value:{total:0,text:'initial',rows:[],trace:0,visits:0}}];
