/// <reference types="node" />
import {ok,strictEqual,notStrictEqual} from "node:assert/strict";
type int=number;
type Measured={length:number|null;};
function workload(n:int,seed:int):int {
    let checksum=0;let i=0;
    while(i<n){
        const kind=(i*13+seed)%6;
        const actual:number|null|undefined=kind===0?null:kind===1?undefined:kind===2?0.0:kind===3?Number.NaN:kind===4?-0.0:7.0;
        if(kind===0){strictEqual(actual,null);}else if(kind===1){strictEqual(actual,undefined);}else{ok(actual!==undefined);}
        if(kind===5){ok(Boolean(actual));}else{ok(!Boolean(actual));}
        if(kind===3){ok(Number.isNaN(actual));}else{strictEqual(actual,actual);}
        if(kind===0){notStrictEqual(actual,undefined);}else if(kind===1){strictEqual(actual,undefined);}else{notStrictEqual(actual,null);}
        if(kind===5){ok(Boolean(actual));}else{ok(!Boolean(actual));}
        if(kind===3){notStrictEqual(actual,0.0);}else{ok(!Number.isNaN(actual));}
        const expectedType=kind===0?"object":kind===1?"undefined":"number";
        strictEqual(typeof actual,expectedType);
        notStrictEqual(typeof actual,"string");
        const numericActual=Number(kind);
        ok(numericActual>-1.0);
        ok(numericActual>=numericActual);
        ok(numericActual<6.0);
        ok(numericActual<=numericActual);
        ok(!(numericActual>6.0));
        ok(!(numericActual>=6.0));
        ok(!(numericActual< -1.0));
        ok(!(numericActual<= -1.0));
        ok(Math.abs(numericActual-numericActual)<0.005);
        ok(Math.abs(numericActual-(numericActual+0.125))>=0.000005);
        const text=kind===0?"😀":"x".repeat(kind);
        const expectedLength=kind===0?2.0:Number(kind);
        strictEqual(text.length,expectedLength);
        notStrictEqual(text.length,expectedLength+1.0);
        const measured:number|null=numericActual;
        const record:Measured={length:measured};
        strictEqual(record.length,numericActual);
        notStrictEqual(record.length,numericActual+1.0);
        const code=actual===null?1:actual===undefined?2:Boolean(actual)?4:8;
        checksum=(checksum*131+code)%1000000007;
        i=i+1;
    }
    return checksum;
}
