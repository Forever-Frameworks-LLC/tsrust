export {};
type int=number;
type Result<T>={kind:"Ok";value:T}|{kind:"Err";error:{code:string;offset?:int}};
const encoder=new TextEncoder();
const decoder=new TextDecoder("utf-8",{fatal:true,ignoreBOM:true});
function firstInvalidOffset(bytes:Uint8Array):int|undefined {
    let i=0;
    while(i<bytes.length) {
        const lead=bytes[i];
        if(lead<128) {i+=1;continue;}
        const width=lead>=194&&lead<=223?2:lead>=224&&lead<=239?3:lead>=240&&lead<=244?4:0;
        if(width===0||i+width>bytes.length)return i;
        for(let j=1;j<width;j+=1) {
            const next=bytes[i+j];
            if(next<128||next>191)return i;
            if(j===1&&((lead===224&&next<160)||(lead===237&&next>159)||(lead===240&&next<144)||(lead===244&&next>143)))return i;
        }
        i+=width;
    }
    return undefined;
}
function fromUtf8(text:string):Result<Uint8Array>{return {kind:"Ok",value:encoder.encode(text)};}
function toUtf8(bytes:Uint8Array):Result<string>{
    try {return {kind:"Ok",value:decoder.decode(bytes)};}
    catch {return {kind:"Err",error:{code:"utf8",offset:firstInvalidOffset(bytes)}};}
}
function textFor(index:int):string {
    if(index===0)return "ASCII";
    if(index===1)return "é😀";
    if(index===2)return "A\u0000B";
    return "\uFEFFz";
}
function fromArray(values:readonly int[]):Result<Uint8Array>{
    for(let i=0;i<values.length;i+=1)if(!Number.isInteger(values[i])||values[i]<0||values[i]>255)return {kind:"Err",error:{code:"byte-range",offset:i}};
    return {kind:"Ok",value:Uint8Array.from(values)};
}
function invalid(values:readonly int[]):int[]{
    const constructed=fromArray(values);
    if(constructed.kind==="Err")return [-1,-1];
    const result=toUtf8(constructed.value);
    return result.kind==="Err"?[result.error.code.length,result.error.offset??-1]:[-1,-1];
}
function workload(n:int,seed:int,input:string):int[][] {
    const output:int[][]=[];
    let i=0;
    while(i<n) {
        const text=textFor((i+seed)%4);
        const encoded=fromUtf8(text);
        if(encoded.kind==="Ok") {
            const bytes=encoded.value;
            const before=bytes.length;
            const result=toUtf8(bytes);
            if(result.kind==="Ok") {
                const decoded=result.value;
                const equal=decoded===text?1:0;
                const bad=invalid([65,195,40]);
                const truncated=invalid([65,226,130]);
                output.push([before,equal,bad[0],bad[1],truncated[0],truncated[1],bytes.length]);
            }else output.push([-1000000]);
        }else output.push([-1000000]);
        i+=1;
    }
    return output;
}
