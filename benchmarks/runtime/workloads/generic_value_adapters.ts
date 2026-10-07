function payloadFactory(start: number) {
    let calls = start;
    function identity<T>(value: T): T {
        calls += 1;
        if (typeof value === "boolean") { if (calls !== start + 3) { throw 99; } }
        return value;
    }
    return identity;
}
function missingFactory(): ((value: number) => Receipt<number>) | undefined { return undefined; }
function scalarFactory() {
    let calls = 0;
    function select<T>(value: T): T { calls += 1; return value; }
    return select;
}
function arrayFactory() {
    let calls = 0;
    function select<T>(value: T): T[] { calls += 1; return [value]; }
    return select;
}
function increment(value: number): number { return value + 1; }
function callbackFactory(start: number) {
    let calls = start;
    function invoke<T>(action: (value: T) => T, value: T): Receipt<T> {
        calls += 1;
        return { value: action(value), calls: calls };
    }
    return invoke;
}
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
function doubleFactory(start: number) {
    let calls = start;
    function select<T>(value: T): Receipt<T> {
        calls += 2;
        return { value: value, calls: calls };
    }
    return select;
}
function inferredFactory(start: number) {
    let calls = start;
    function select<T>(left: T, right: T) {
        calls += 1;
        if (calls === start + 1) { return left; }
        return right;
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
function defaultFactory() {
    let calls = 0;
    function next(): number { calls += 1; return calls; }
    function select<T>(value: T, offset: number = next()): Receipt<T> {
        return { value: value, calls: offset };
    }
    return select;
}
function restFactory() {
    let calls = 0;
    function select<T>(fallback: T, ...values: T[]): Receipt<T> {
        calls += 1;
        return { value: fallback, calls: calls + values.length };
    }
    return select;
}
function genericFactory<T>(seed: T, start: number) {
    let calls = start;
    function select<U>(value: U): Receipt<U> {
        calls += 1;
        if (typeof seed === "string") { calls += 1; }
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
        const defaults = defaultFactory();
        let defaultValue = 0;
        let omittedCalls = 0;
        let overrideCalls = 0;
        let undefinedCalls = 0;
        {
            const view: (value: number, offset?: number) => Receipt<number> = defaults;
            const omitted = view(base);
            defaultValue = omitted.value;
            omittedCalls = omitted.calls;
            overrideCalls = view(base, 9).calls;
            undefinedCalls = view(base, undefined).calls;
        }
        const defaultsAfter = defaults("after");
        const rest = restFactory();
        let emptyValue = 0;
        let emptyCalls = 0;
        let packedCalls = 0;
        {
            const view: (fallback: number, ...values: number[]) => Receipt<number> = rest;
            const empty = view(base);
            emptyValue = empty.value;
            emptyCalls = empty.calls;
            packedCalls = view(base + 1, base + 2, base + 3).calls;
        }
        const restAfter = rest("after", "one");
        const direct: (value: number) => Receipt<number> = make(base + 40);
        const directFirst = direct(base + 5);
        const directSecond = direct(base + 6);
        const factoryMapped = [base + 7, base + 8].map(make(base + 50));
        const factory = make;
        const alias: (value: number) => Receipt<number> = factory(base + 60);
        const aliasValue = alias(base + 9);
        const selectedMapped = [base + 11, base + 12].map((false ? factory : factory)(base + 70));
        const genericDirect: (value: number) => Receipt<number> = genericFactory("string", base + 80);
        const genericFirst = genericDirect(base + 13);
        const genericSecond = genericDirect(base + 14);
        const genericMapped = [base + 15, base + 16].map(genericFactory(base, base + 90));
        let selections = 0;
        let evaluations = 0;
        const distinct: (value: number) => Receipt<number> = (selections++ === 0 ? make : doubleFactory)(base + 100 + evaluations++);
        const distinctFirst = distinct(base + 17);
        const distinctSecond = distinct(base + 18);
        const distinctMapped = [base + 19, base + 20].map((selections++ === 0 ? make : doubleFactory)(base + 110 + evaluations++));
        let nestedSelections = 0;
        let nestedEvaluations = 0;
        const nested: (value: number) => Receipt<number> = (nestedSelections++ === 0 ? (nestedSelections++ === 1 ? make : doubleFactory) : doubleFactory)(base + 120 + nestedEvaluations++);
        const nestedFirst = nested(base + 21);
        const nestedSecond = nested(base + 22);
        const nestedMapped = [base + 23, base + 24].map((nestedSelections++ === 0 ? make : (nestedSelections++ === 3 ? doubleFactory : make))(base + 130 + nestedEvaluations++));
        const inferred: (left: number, right: number) => number = inferredFactory(base + 140);
        const inferredFirst = inferred(base + 25, base + 26);
        const inferredSecond = inferred(base + 27, base + 28);
        const inferredMapped = [base + 29, base + 30].map(inferredFactory(base + 150));
        const freshHolder: Holder = { run: make(base + 160) };
        const freshRecord = freshHolder.run(base + 31);
        const freshActions: ((value: string) => Receipt<string>)[] = [make(base + 170)];
        const freshStored = freshActions[0]("fresh");
        const callback: (action: (value: number) => number, value: number) => Receipt<number> = callbackFactory(base + 180);
        const callbackFirst = callback(increment, base + 32);
        const callbackSecond = callback(increment, base + 33);
        const nullable: ((value: number) => Receipt<number>) | undefined = make(base + 190);
        const nullableRecord = nullable?.(base + 35);
        if (nullableRecord === undefined) { throw 99; }
        const absent = missingFactory();
        let skipped = 0;
        const absentValue = absent?.(base + skipped++);
        if (skipped !== 0) { throw 99; }
        const scalarView: ((value: number) => number | number[]) = (true ? scalarFactory : arrayFactory)();
        const arrayView: ((value: number) => number | number[]) = (false ? scalarFactory : arrayFactory)();
        const scalarResult = scalarView(base + 37);
        const arrayResult = arrayView(base + 38);
        if (typeof scalarResult !== "number") { throw 99; }
        if (typeof arrayResult === "number") { throw 99; }
        const payload = payloadFactory(base + 200);
        let payloadInt = 0;
        let payloadString = 0;
        {
            const view: (value: (value: number) => number) => (value: number) => number = payload;
            const action = view((value: number): number => value + 1);
            payloadInt = action(base + 39);
        }
        {
            const view: (value: (value: string) => string) => (value: string) => string = payload;
            const action = view((value: string): string => value + "!");
            payloadString = action(String(base)).length;
        }
        const payloadAfter = payload(true);
        output.push([firstValue, firstCalls, generic.value.length, generic.calls, last.value.length, last.calls,
            mapped[0].value, mapped[1].value, mapped[0].calls, mapped[1].calls, after.value ? 1 : 0, after.calls,
            record.value, record.calls, stored.value.length, stored.calls, input.length,
            reduced, reducedAfter.length, reversed, reversedAfter.length, found, searchedAfter ? 1 : 0,
            sortable[0], copied[0], sortedAfter,
            defaultValue, omittedCalls, overrideCalls, undefinedCalls, defaultsAfter.calls, defaultsAfter.value.length,
            emptyValue, emptyCalls, packedCalls, restAfter.calls, restAfter.value.length,
            directFirst.value, directFirst.calls, directSecond.value, directSecond.calls,
            factoryMapped[0].value, factoryMapped[1].value, factoryMapped[0].calls, factoryMapped[1].calls,
            aliasValue.value, aliasValue.calls, selectedMapped[0].value, selectedMapped[1].value,
            selectedMapped[0].calls, selectedMapped[1].calls,
            genericFirst.value, genericFirst.calls, genericSecond.calls, genericMapped[0].value, genericMapped[1].value,
            genericMapped[0].calls, genericMapped[1].calls,
            distinctFirst.value, distinctFirst.calls, distinctSecond.calls,
            distinctMapped[0].value, distinctMapped[1].value, distinctMapped[0].calls, distinctMapped[1].calls,
            selections, evaluations,
            nestedFirst.value, nestedFirst.calls, nestedSecond.calls,
            nestedMapped[0].value, nestedMapped[1].value, nestedMapped[0].calls, nestedMapped[1].calls,
            nestedSelections, nestedEvaluations,
            inferredFirst, inferredSecond, inferredMapped[0], inferredMapped[1],
            freshRecord.value, freshRecord.calls, freshStored.value.length, freshStored.calls,
            callbackFirst.value, callbackFirst.calls, callbackSecond.value, callbackSecond.calls,
            nullableRecord.value, nullableRecord.calls, absentValue === undefined ? 1 : 0, scalarResult, arrayResult[0],
            payloadInt, payloadString, payloadAfter ? 1 : 0]);
    }
    return output;
}
function main() {}
