type int = number;
type Trace = { code: int; };
function receiver(trace: Trace): int { trace.code = trace.code * 10 + 1; return -35; }
function radix(trace: Trace): int { trace.code = trace.code * 10 + 2; return 16; }
function main() {
    const trace: Trace = { code: 0 };
    const result: string = receiver(trace).toString(radix(trace));
    console.log(result); console.log(trace.code);
    const decimal: string = receiver(trace).toString();
    console.log(decimal); console.log(trace.code);
    const reusable: int = -255;
    console.log(reusable.toString(2)); console.log(reusable);
}
