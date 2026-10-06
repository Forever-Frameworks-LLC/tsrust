type int = number;
type Trace = { code: int; };
function receiver(trace: Trace): string { trace.code = trace.code * 10 + 1; return "<é😀&>"; }
function attribute(trace: Trace): string { trace.code = trace.code * 10 + 2; return "a\"&<>"; }
function main() {
    const trace: Trace = { code: 0 };
    const anchor: string = receiver(trace).anchor(attribute(trace)); console.log(trace.code); console.log(anchor);
    trace.code = 0;
    const color: string = receiver(trace).fontcolor(attribute(trace)); console.log(trace.code); console.log(color);
    trace.code = 0;
    const size: string = receiver(trace).fontsize(attribute(trace)); console.log(trace.code); console.log(size);
    trace.code = 0;
    const link: string = receiver(trace).link(attribute(trace)); console.log(trace.code); console.log(link);
    trace.code = 0;
    const bold: string = receiver(trace).bold(); console.log(trace.code); console.log(bold);
}
