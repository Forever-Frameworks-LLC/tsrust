export {};
type int = number;
type Bytes = { data: Uint8Array };
type Result<T> = {kind:"Ok";value:T} | {kind:"Err";error:{code:string;offset?:int}};
function fromArray(values: readonly int[]): Result<Bytes> {
    for(let i=0;i<values.length;i+=1) if(!Number.isInteger(values[i])||values[i]<0||values[i]>255) return {kind:"Err",error:{code:"byte-range",offset:i}};
    return {kind:"Ok",value:{data:Uint8Array.from(values)}};
}
function append(bytes: Bytes, values: readonly int[]): Result<void> {
    const size = bytes.data.length + values.length;
    if(!Number.isSafeInteger(size)) return {kind:"Err",error:{code:"length-range"}};
    for(let i=0;i<values.length;i+=1) if(!Number.isInteger(values[i])||values[i]<0||values[i]>255) return {kind:"Err",error:{code:"byte-range",offset:i}};
    if(values.length>0) { const next = new Uint8Array(size); next.set(bytes.data); next.set(values,bytes.data.length); bytes.data=next; }
    return {kind:"Ok",value:undefined};
}
function copy(bytes: Bytes, start: int, end: int): Result<Bytes> {
    if(start<0||start>bytes.data.length) return {kind:"Err",error:{code:"range",offset:start}};
    if(end<start||end>bytes.data.length) return {kind:"Err",error:{code:"range",offset:end}};
    return {kind:"Ok",value:{data:bytes.data.slice(start,end)}};
}
function length(bytes: Bytes): int {return bytes.data.length;}
function read(bytes: Bytes,index:int):int {return index>=0&&index<bytes.data.length?bytes.data[index]:-1;}
function write(bytes:Bytes,index:int,value:int):int {if(index<0||index>=bytes.data.length||value<0||value>255)return -1;bytes.data[index]=value;return 1;}
function workload(n:int,seed:int,input:string):int[][] {
    const output: int[][] = [];
    const add = append;
    const duplicate = copy;
    let i = 0;
    while (i < n) {
        const byte = (i + seed) % 256;
        const result = fromArray([byte,65]);
        switch(result.kind) {case "Ok": {
            const bytes = result.value;
            const initial = length(bytes);
            const added = add(bytes,[255,7]);
            const success = added.kind === "Ok" ? 1 : -2;
            const failed = append(bytes,[8,256]);
            const invalid = failed.kind === "Err" ? failed.error.code.length : -2;
            const offset = failed.kind === "Err" ? failed.error.offset ?? -3 : -1;
            const preserved = read(bytes,3);
            const copied = duplicate(bytes,0,2);
            switch(copied.kind) {case "Ok": {
                const part = copied.value;
                const ownerWrite = write(bytes,0,(byte + 1) % 256);
                const independent = read(part,0);
                const partWrite = write(part,1,9);
                const ownerRetained = read(bytes,1);
                const endCopy = copy(bytes,4,4);
                const empty = endCopy.kind === "Ok" ? length(endCopy.value) : -2;
                const range = copy(bytes,0,5);
                const rangeOffset = range.kind === "Err" ? range.error.offset ?? -3 : -2;
                output.push([initial,success,invalid,offset,preserved,length(part),ownerWrite,independent,partWrite,ownerRetained,empty,rangeOffset,length(bytes)]);
                break;
            } case "Err": output.push([-1000000]);break;}
            break;
        }case "Err":output.push([-1000000]);break;}
        i += 1;
    }
    return output;
}
