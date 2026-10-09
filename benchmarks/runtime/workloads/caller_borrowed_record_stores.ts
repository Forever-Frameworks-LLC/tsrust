type NullableHolder = { run: (() => int) | undefined; };

type OnceHolder = { run: () => int; };

type int = number;
type Holder = {
    run: () => int;
};

type MutableHolder = {
    run: () => int;
};

type Box = {
    handler: Holder;
};

function workload(n: int, seed: int, input: string): int[][] {
    const output: int[][] = [];
    let i = 0;
    while (i < n) {
        const value = i % 5 + seed % 5 + 1;
        const base = [value];
        const other = [value + 2];
        let first = 0;
        let replaced = 0;
        let nested = 0;
        let callback = 0;
        let callbackReplaced = 0;
        let pairReplaced = 0;
        let paired = 0;
        let changedFirst = 0;
        let changedSecond = 0;
        let callbackFirst = 0;
        let callbackSecond = 0;
        let callbackAfter = 0;
        let mutableReplaced = 0;
        {
            const holder: Holder = {
                run: (): int => 0
            };
            installArray(holder, base);
            first = holder.run();
            replaceBoth(holder, base, other);
            replaced = holder.run();
            const box: Box = {
                handler: {
                    run: (): int => 0
                }
            };
            installNested(box, base, other);
            nested = box.handler.run();
            const action = (): int => base[0];
            const constant = (): int => 9;
            const callbackHolder: Holder = {
                run: (): int => 0
            };
            installCallback(callbackHolder, action);
            callback = callbackHolder.run();
            installDirect(callbackHolder, constant);
            const readCallback = (): int => callbackHolder.run();
            callbackReplaced = readCallback();
            const pairHolder: Holder = {
                run: (): int => 0
            };
            installPair(pairHolder, action, constant);
            paired = pairHolder.run();
            installPair(pairHolder, constant, action);
            pairReplaced = pairHolder.run();
        }

        base[0] += 1;
        {
            const holder: MutableHolder = {
                run: (): int => 0
            };
            installMutable(holder, base);
            changedFirst = holder.run();
            changedSecond = holder.run();
        }

        {
            const action = (): int => { base[0] += 1; return base[0]; };
            const secondAction = (): int => { other[0] += 10; return other[0]; };
            {
                const holder: MutableHolder = { run: (): int => 0 };
                installMutableCallback(holder, action);
                callbackFirst = holder.run();
                callbackSecond = holder.run();
                installMutableCallback(holder, secondAction);
                const invoke = (): int => holder.run();
                mutableReplaced = invoke();
            }
            callbackAfter = action();
        }
        let consumed = 0;
        let afterFailure = 0;
        let beforeFailure = 0;
        {
            const owned = [value];
            const once = (): int => { const moved = owned; return moved[0]; };
            const holder: OnceHolder = { run: (): int => 0 };
            installOnce(holder, once);
            consumed = holder.run();
        }
        {
            const action = (): int => base[0];
            const holder: Holder = { run: (): int => value + 9 };
            let caught = false;
            try { installAndFail(holder, action); } catch (error) { caught = error === "installed"; }
            afterFailure = holder.run() + (caught ? 0 : 100);
        }
        {
            const action = (): int => base[0];
            const holder: Holder = { run: (): int => value + 9 };
            let caught = false;
            try { failBeforeInstall(holder, action, true); } catch (error) { caught = error === "before"; }
            beforeFailure = holder.run() + (caught ? 0 : 100);
        }

        let callbackFailure = 0;
        {
            const payload = [value + 17];
            const action = (): int => { throw payload[0]; };
            const holder: Holder = { run: (): int => 0 };
            installDirect(holder, action);
            const guarded = (): int => {
                try { return holder.run(); } catch (error) {
                    return typeof error === "number" ? error : 0;
                }
            };
            callbackFailure = guarded();
        }

        let nullableBefore = 0;
        let nullableAfter = 0;
        let nullableEmpty = 0;
        {
            const data = [value];
            const action = (): int => { data[0] += 1; return data[0]; };
            const holder: NullableHolder = { run: undefined };
            installNullable(holder, action);
            nullableBefore = holder.run!();
            relayClearNullable(holder, i % 2 === 0);
            nullableAfter = action();
            nullableEmpty = holder.run === undefined ? 1 : 0;
        }

        let nullableReleased = 0;
        let nullableReplacement = 0;
        let nullableReplacementAfter = 0;
        {
            const originalData = [value];
            const replacementData = [value + 4];
            const originalAction = (): int => { originalData[0] += 1; return originalData[0]; };
            const replacementAction = (): int => { replacementData[0] += 3; return replacementData[0]; };
            const holder: NullableHolder = { run: undefined };
            installNullable(holder, originalAction);
            installNullable(holder, replacementAction);
            nullableReleased = originalAction();
            nullableReplacement = holder.run!();
            clearNullable(holder);
            nullableReplacementAfter = replacementAction() + (holder.run === undefined ? 0 : 100);
        }

        let factoryEffects = 0;
        let factoryBefore = 0;
        let factoryReleased = 0;
        let factoryReplacement = 0;
        let factoryAfter = 0;
        {
            const original = counter(value);
            const replacement = counter(value + 4);
            const holder: NullableHolder = { run: undefined };
            const effects = [0];
            installWithEffects(holder, original, effects);
            factoryBefore = holder.run!();
            installWithEffects(holder, replacement, effects);
            factoryEffects = effects[0];
            factoryReleased = original();
            factoryReplacement = holder.run!();
            clearNullable(holder);
            factoryAfter = replacement() + (holder.run === undefined ? 0 : 100);
        }

        let dataEffects = 0;
        let dataReleased = 0;
        let dataReplacement = 0;
        let dataAfter = 0;
        {
            const original = [value];
            const replacement = [value + 4];
            const holder: NullableHolder = { run: undefined };
            const dataCalls = [0];
            installNullableData(holder, original, dataCalls);
            holder.run!();
            installNullableData(holder, replacement, dataCalls);
            dataEffects = dataCalls[0];
            original[0] += 1;
            dataReleased = original[0];
            dataReplacement = holder.run!();
            clearNullable(holder);
            replacement[0] += 1;
            dataAfter = replacement[0] + (holder.run === undefined ? 0 : 100);
        }

        let conditionalFirst = 0;
        let conditionalReleased = 0;
        let conditionalReplacement = 0;
        let conditionalEffects = 0;
        {
            const original = [value];
            const replacement = [value + 4];
            const calls = [0];
            const holder: NullableHolder = { run: undefined };
            installConditional(holder, original, i % 2 === 0, calls);
            conditionalFirst = holder.run!();
            installConditional(holder, replacement, i % 2 !== 0, calls);
            original[0] += 1;
            conditionalReleased = original[0];
            conditionalReplacement = holder.run!();
            conditionalEffects = calls[0];
        }
        output.push([first, replaced, nested, callback, paired, changedFirst, changedSecond, callbackFirst, callbackSecond, callbackAfter, base[0], callbackReplaced, pairReplaced, mutableReplaced, consumed, afterFailure, beforeFailure, callbackFailure, nullableBefore, nullableAfter, nullableEmpty, nullableReleased, nullableReplacement, nullableReplacementAfter, factoryBefore, factoryReleased, factoryReplacement, factoryAfter, dataReleased, dataReplacement, dataAfter, factoryEffects, dataEffects, conditionalFirst, conditionalReleased, conditionalReplacement, conditionalEffects]);
        i += 1;
    }

    return output;
}

function installArray(holder: Holder, source: readonly int[]): void {
    const alias = source;
    holder.run = (): int => alias[0];
}

function replaceBoth(holder: Holder, left: readonly int[], right: readonly int[]): void {
    installArray(holder, left);
    installArray(holder, right);
}

function installNested(box: Box, left: readonly int[], right: readonly int[]): void {
    box.handler.run = (): int => left[0] + right[0];
}

function installCallback(holder: Holder, action: () => int): void {
    const alias = action;
    holder.run = (): int => alias();
}

function installPair(holder: Holder, left: () => int, right: () => int): void {
    holder.run = (): int => left() + right();
}

function installMutable(holder: MutableHolder, source: int[]): void {
    holder.run = (): int => {
        source[0] += 1;
        return source[0];
    };
}

function installMutableCallback(holder: MutableHolder, action: () => int): void {
    holder.run = (): int => action();
}

function installDirect(holder: Holder, action: () => int): void {
    holder.run = action;
}

function installOnce(holder: OnceHolder, action: () => int): void { holder.run = action; }

function installAndFail(holder: Holder, action: () => int): void { holder.run = action; throw "installed"; }

function failBeforeInstall(holder: Holder, action: () => int, fail: boolean): void { if (fail) { throw "before"; } holder.run = action; }

function main() {
}

function installNullable(holder: NullableHolder, action: () => int): void {
    holder.run = action;
}
function clearNullable(holder: NullableHolder): void { holder.run = undefined; }
function relayClearNullable(holder: NullableHolder, early: boolean): void {
    if (early) { clearNullable(holder); return; }
    clearNullable(holder);
}

function counter(start: int): () => int {
    let value = start;
    return (): int => { value += 1; return value; };
}
function installNullableData(holder: NullableHolder, source: int[], calls: int[]): void {
    calls[0] += 1;
    holder.run = (): int => { source[0] += 1; return source[0]; };
}

function installWithEffects(holder: NullableHolder, action: () => int, calls: int[]): void {
    calls[0] += 1;
    installNullable(holder, action);
}

function installConditional(holder: NullableHolder, source: int[], plus: boolean, calls: int[]): void {
    calls[0] += 1;
    holder.run = chooseCallbackBranch(plus, calls) ? (calls[0] += 100, (): int => source[0] + 1) : (calls[0] += 200, (): int => source[0] + 2);
}
function chooseCallbackBranch(plus: boolean, calls: int[]): boolean {
    calls[0] += 10;
    return plus;
}
