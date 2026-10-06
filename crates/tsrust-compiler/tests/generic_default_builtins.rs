use tsrust_compiler::check;

#[test]
fn unused_generic_defaults_reject_known_builtin_result_families() {
    let methods = [
        "items.join(\",\")",
        "items.toString()",
        "items.slice()",
        "items.concat(items)",
        "items.filter((value:int):boolean=>{return true;})",
        "items.toReversed()",
        "items.toSorted()",
        "items.with(0,1)",
        "items.toSpliced(0,1)",
        "items.splice(0,1)",
        "items.flat()",
        "items.at(0)",
        "items.pop()",
        "items.shift()",
        "items.remove(0)",
        "items.unshift(1)",
        "items.includes(1)",
        "items.indexOf(1)",
        "items.lastIndexOf(1)",
        "items.findIndex((value:int):boolean=>{return true;})",
        "items.findLastIndex((value:int):boolean=>{return true;})",
        "items.some((value:int):boolean=>{return true;})",
        "items.every((value:int):boolean=>{return true;})",
        "items.find((value:int):boolean=>{return true;})",
        "items.findLast((value:int):boolean=>{return true;})",
        "items.map((value:int):int=>{return value;})",
        "items.flatMap((value:int):int[]=>{return [value];})",
        "items.reduce((acc:int,value:int):int=>{return acc+value;},0)",
        "items.reduceRight((acc:int,value:int):int=>{return acc+value;},0)",
        "items.push(1)",
        "items.reverse()",
        "items.sort()",
        "items.fill(1)",
        "items.copyWithin(0,1)",
        "items.forEach((value:int):void=>{return;})",
        "items.length",
        "text.padStart(8)",
        "text.padEnd(8)",
        "text.split(\",\")",
        "text.substr(1)",
        "text.slice()",
        "text.substring(1)",
        "text.charAt(0)",
        "text.repeat(2)",
        "text.concat(\"x\")",
        "text.replace(\"a\",\"b\")",
        "text.replaceAll(\"a\",\"b\")",
        "text.trim()",
        "text.trimStart()",
        "text.trimEnd()",
        "text.trimLeft()",
        "text.trimRight()",
        "text.toLowerCase()",
        "text.toUpperCase()",
        "text.toWellFormed()",
        "text.isWellFormed()",
        "text.includes(\"a\")",
        "text.startsWith(\"a\")",
        "text.endsWith(\"a\")",
        "text.indexOf(\"a\")",
        "text.lastIndexOf(\"a\")",
        "text.toString()",
        "text.valueOf()",
        "text.bold()",
        "text.anchor(\"id\")",
        "text.length",
        "number.toFixed(2)",
        "number.toExponential()",
        "number.toPrecision(2)",
        "number.valueOf()",
        "integer.toString()",
        "integer.valueOf()",
        "flag.toString()",
        "flag.valueOf()",
        "Math.abs(number)",
        "Math.PI",
        "Number.NaN",
        "Number.parseFloat(text)",
        "Number.isFinite(number)",
        "Array.isArray(items)",
        "String(integer)",
        "Boolean(items)",
        "Number(integer)",
        "clone(items)",
        "console.log(integer)",
        "Array.of<int>(1)",
        "text?.padStart(8)",
        "maybe?.split(\",\")",
        "items?.slice()",
    ];
    for expression in methods {
        let source = format!(
            "function bad<T>(items:int[],text:string,number:number,integer:int,flag:boolean,maybe:string|null,value:T={expression}):T{{return value;}}function main(){{}}"
        );
        let error = check(&source).err().expect(expression);
        assert_eq!(error.code, "type", "{expression}: {error:?}");
        assert!(
            error.message.contains("every type permitted"),
            "{expression}: {error:?}"
        );
        assert_eq!(
            &source[error.span.start..error.span.end],
            expression,
            "{error:?}"
        );
    }
}

#[test]
fn generic_dependent_builtin_and_declared_callback_results_keep_their_shapes() {
    let source = r#"
function same<T>(value:T,fallback:T=value):T{return fallback;}
function slice<T>(items:T[],fallback:T[]=items.slice()):T[]{return fallback;}
function optionalSlice<T>(items:T[],fallback:T[]=items?.slice()):T[]{return fallback;}
function map<T>(items:int[],factory:(value:int)=>T,fallback:T[]=items.map(factory)):T[]{return fallback;}
function flatMap<T>(items:int[],factory:(value:int)=>T[],fallback:T[]=items.flatMap(factory)):T[]{return fallback;}
function flat<T>(items:T[][],fallback:T[]=items.flat()):T[]{return fallback;}
function minFlat<T>(items:T[][],fallback:T[][]=items.flat(-9223372036854775808)):T[][]{return fallback;}
function noFlat<T>(items:T[][],fallback:T[][]=items.flat(-1)):T[][]{return fallback;}
function allFlat<T>(items:T[][][],fallback:T[]=items.flat(99)):T[]{return fallback;}
function reduce<T>(items:int[],seed:T,factory:(acc:T,value:int)=>T,fallback:T=items.reduce(factory,seed)):T{return fallback;}
function pick<T>(items:T[],fallback:T|undefined=items.at(0)):T|undefined{return fallback;}
function produce<T>(factory:()=>T,fallback:T=factory()):T{return fallback;}
type Box<T>={concat:()=>T;};
function member<T>(box:Box<T>,fallback:T=box.concat()):T{return fallback;}
function main(){console.log(same<int>(3));const values:int[]=[1,2];console.log(slice<int>(values).join(","));}
"#;
    check(source).unwrap();
}
