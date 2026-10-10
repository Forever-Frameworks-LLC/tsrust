export {};
type int = number;
type Bytes = Uint8Array;
type Holder = {bytes:Bytes};
function read(bytes:Bytes,index:int):int {return index>=0&&index<bytes.length?bytes[index]:-1;}
function write(bytes:Bytes,index:int,value:int):void {if(index<0||index>=bytes.length||value<0||value>255)throw Error("invalid write");bytes[index]=value;}
function length(bytes:Bytes):int {return bytes.length;}
function cloneBytes(bytes:Bytes):Bytes {return bytes.slice();}
function cloneRows(rows:readonly Holder[]):Holder[] {return rows.map(row=>({bytes:cloneBytes(row.bytes)}));}
function workload(n:int,seed:int,input:string):int[][] {
    const output: int[][] = [];
    let i = 0;
    while(i<n) {
        const byte = (i + seed) % 256;
        const bytes = new Uint8Array(3).fill(byte);
        const duplicate = cloneBytes(bytes);
        const rows: Holder[] = [{bytes:bytes}];
        const saved = cloneRows(rows);
        write(rows[0].bytes,0,(byte + 1) % 256);
        write(duplicate,1,7);
        output.push([read(duplicate,0),read(duplicate,1),read(saved[0].bytes,0),read(saved[0].bytes,1),read(rows[0].bytes,0),length(duplicate),length(saved[0].bytes)]);
        i += 1;
    }
    return output;
}
