type int = number;
type Trace = { calls: int; arguments: int; fallbacks: int; factories: int; };
function argument(trace: Trace, value: int): int { trace.arguments = trace.arguments + 1; return value; }
function step(input: int, trace: Trace): int { trace.calls = trace.calls + 1; return input * 17 + 3; }
function fallbackStep(input: int, trace: Trace): int { trace.calls = trace.calls + 1; return input * 17 + 103; }
function chooseRead(tag: int): ((input: int, trace: Trace) => int) | null | undefined {
    if (tag === 1) { return null; } if (tag === 2) { return undefined; } return step;
}
function makeBase(trace: Trace, tag: int): ((input: int, trace: Trace) => int) | null | undefined {
    trace.factories = trace.factories + 1; return chooseRead(tag);
}
function makeFallback(trace: Trace): (input: int, trace: Trace) => int {
    trace.fallbacks = trace.fallbacks + 1; return step;
}
function chooseOwned(tag: int): ((input: int, trace: Trace) => int) | undefined {
    if (tag === 2) { return undefined; } return step;
}
function useRead(action: ((input: int, trace: Trace) => int) | null, value: int, trace: Trace): int {
    if (action !== null) { return action(value, trace); } return 0;
}
function consumeText(text: string): string { return text; }
function main() {
    const trace: Trace = { calls: 0, arguments: 0, fallbacks: 0, factories: 0 };
    const absent = makeBase(trace, 1)?.(argument(trace, 7), trace);
    console.log(typeof absent, trace.factories, trace.arguments, trace.calls);
    const present = makeBase(trace, 0)?.(argument(trace, 4), trace);
    if (present !== undefined) { console.log(present, trace.factories, trace.arguments, trace.calls); }
    const fallback = chooseRead(2) ?? makeFallback(trace);
    console.log(fallback(argument(trace, 5), trace), trace.fallbacks, trace.arguments, trace.calls);
    const kept = chooseRead(0) ?? makeFallback(trace);
    console.log(kept(argument(trace, 6), trace), trace.fallbacks, trace.arguments, trace.calls);
    let action: ((input: int, trace: Trace) => int) | null | undefined = null;
    const assigned = action ??= makeFallback(trace);
    const first = assigned(argument(trace, 7), trace);
    const second = action(argument(trace, 8), trace);
    console.log(first, second, trace.fallbacks, trace.arguments, trace.calls);
    let owner: ((input: int, trace: Trace) => int) | undefined = chooseOwned(2);
    owner ??= step;
    if (owner !== undefined) { console.log(owner(argument(trace, 9), trace), trace.arguments, trace.calls); }
    const offset = 2;
    const capture: ((input: int, callTrace: Trace) => int) | null = (callValue: int, callTrace: Trace): int => {
        callTrace.calls = callTrace.calls + 1; return callValue + offset;
    };
    console.log(useRead(capture, argument(trace, 10), trace), trace.arguments, trace.calls);
    const text = "abc".repeat(1);
    const once: ((input: int, callTrace: Trace) => int) | null = (callValue: int, callTrace: Trace): int => {
        callTrace.calls = callTrace.calls + 1; return callValue + consumeText(text).length;
    };
    if (once !== null) { console.log(once(argument(trace, 11), trace), trace.arguments, trace.calls); }
}

main();
