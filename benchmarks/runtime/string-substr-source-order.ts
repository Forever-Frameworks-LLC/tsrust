type int = number;
interface String { substr(from?: number, length?: number): string; }
type Trace = { code: int; };
function receiver(trace: Trace): string { trace.code = trace.code * 10 + 1; return "é😀Z"; }
function begin(trace: Trace, value: int): int { trace.code = trace.code * 10 + 2; return value; }
function cap(trace: Trace, value: int): int { trace.code = trace.code * 10 + 3; return value; }
function main() {
    const trace: Trace = { code: 0 };
    const partial: string = receiver(trace).substr(begin(trace, -3), cap(trace, 2));
    console.log(trace.code); console.log(partial);
    trace.code = 0;
    const empty: string = receiver(trace).substr(begin(trace, 1), cap(trace, 0));
    console.log(trace.code); console.log(empty);
    console.log("é😀Z".substr());
    console.log("é😀Z".substr(1));
    const owned: string = "é😀".repeat(2);
    console.log(owned.substr(0, 1)); console.log(owned);
    console.log("abc".substr(-1000000, 1000000));
    console.log("abc".substr(1000000, 1000000));
}
