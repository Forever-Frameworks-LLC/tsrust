type Receipt<T> = { value: T; calls: number; };
type Holder = { run: (value: number) => Receipt<number>; };
function make(start: number) {
    let calls = start;
    function select<T>(value: T): Receipt<T> {
        calls += 1;
        return { value: value, calls: calls };
    }
    return select;
}
function workload(n: number, seed: number, input: string): number[][] {
    const output: number[][] = [];
    for (let i = 0; i < n; i += 1) {
        const base = i % 5 + seed % 5 + 1;
        const action = make(base);
        let firstValue = 0;
        let firstCalls = 0;
        {
            const view: (value: number) => Receipt<number> = action;
            const first = view(base);
            firstValue = first.value;
            firstCalls = first.calls;
        }
        const generic = action("generic");
        const owned: (value: string) => Receipt<string> = action;
        const last = owned("owner");
        const mapper = make(base + 10);
        const mapped = [base, base + 1].map(mapper);
        const after = mapper(true);
        const recordAction = make(base + 20);
        const holder: Holder = { run: recordAction };
        const record = holder.run(base + 2);
        const arrayAction = make(base + 30);
        const actions: ((value: string) => Receipt<string>)[] = [arrayAction];
        const stored = actions[0]("array");
        output.push([firstValue, firstCalls, generic.value.length, generic.calls, last.value.length, last.calls,
            mapped[0].value, mapped[1].value, mapped[0].calls, mapped[1].calls, after.value ? 1 : 0, after.calls,
            record.value, record.calls, stored.value.length, stored.calls, input.length]);
    }
    return output;
}
function main() {}
