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
function reducer() {
    let calls = 0;
    function first<T>(left: T, right: T): T {
        calls += 1;
        return calls === 4 ? right : left;
    }
    return first;
}
function predicate() {
    let calls = 0;
    function accept<T>(value: T): boolean {
        calls += 1;
        return calls === 2;
    }
    return accept;
}
function comparator() {
    let seen = 0;
    function compare<T>(left: T, right: T): number {
        if (typeof left === "string") { return seen; }
        seen = 1;
        return 0;
    }
    return compare;
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
        const reduce = reducer();
        const reduced = [base, base + 1, base + 2].reduce(reduce, base + 3);
        const reducedAfter = reduce("before", "fourth");
        const reduceRight = reducer();
        const reversed = [base, base + 1, base + 2].reduceRight(reduceRight, base + 4);
        const reversedAfter = reduceRight("before", "fourth");
        const accept = predicate();
        const found = [base, base + 1, base + 2].findIndex(accept);
        const searchedAfter = accept("after");
        const compare = comparator();
        const sortable = [base + 2, base, base + 1];
        sortable.sort(compare);
        const copied = sortable.toSorted(compare);
        const sortedAfter = compare("after", "unused");
        output.push([firstValue, firstCalls, generic.value.length, generic.calls, last.value.length, last.calls,
            mapped[0].value, mapped[1].value, mapped[0].calls, mapped[1].calls, after.value ? 1 : 0, after.calls,
            record.value, record.calls, stored.value.length, stored.calls, input.length,
            reduced, reducedAfter.length, reversed, reversedAfter.length, found, searchedAfter ? 1 : 0,
            sortable[0], copied[0], sortedAfter]);
    }
    return output;
}
function main() {}
