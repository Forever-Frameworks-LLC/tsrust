type int = number;
type Trace = { code: int; };
function input(trace: Trace, value: string): string { trace.code = trace.code * 10 + 1; return value; }
function main() {
    const trace: Trace = { code: 0 };
    trace.code = 0;
    console.log(Number.parseFloat(input(trace, "1.5tail")) === 1.5); console.log(trace.code);
    trace.code = 0;
    console.log(Number.parseFloat(input(trace, "1e+tail")) === 1.0); console.log(trace.code);
    trace.code = 0;
    console.log(1.0 / Number.parseFloat(input(trace, "-0tail")) < 0.0); console.log(trace.code);
    trace.code = 0;
    console.log(Number.parseFloat(input(trace, "Infinitytail")) > 1.0e308); console.log(trace.code);
    trace.code = 0;
    console.log(Number.parseFloat(input(trace, "-Infinity!")) < -1.0e308); console.log(trace.code);
    trace.code = 0;
    console.log(Number.parseFloat(input(trace, "\uFEFF.5")) === 0.5); console.log(trace.code);
    trace.code = 0;
    console.log(Number.parseFloat(input(trace, "0x1")) === 0.0); console.log(trace.code);
    trace.code = 0;
    const invalid: number = Number.parseFloat(input(trace, "\u00851"));
    console.log(invalid !== invalid); console.log(trace.code);
    const reusable: string = "-0";
    console.log(1.0 / Number.parseFloat(reusable) < 0.0); console.log(reusable);
}
