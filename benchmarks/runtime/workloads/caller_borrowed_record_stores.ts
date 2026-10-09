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

        output.push([first, replaced, nested, callback, paired, changedFirst, changedSecond, callbackFirst, callbackSecond, callbackAfter, base[0], callbackReplaced, pairReplaced, mutableReplaced, consumed, afterFailure, beforeFailure, callbackFailure, nullableBefore, nullableAfter, nullableEmpty]);
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
