import { strictEqual } from 'node:assert';

type Holder = { run: () => number };
function install(holder: Holder, action: () => number): void {
    holder.run = action;
}
function aliasInstall(holder: Holder, action: () => number): void {
    const alias = action;
    holder.run = alias;
}
function relay(action: () => number, holder: Holder): void {
    aliasInstall(holder, action);
}

const values = [3];
{
    const action = () => values[0];
    const holder: Holder = { run: () => 0 };
    install(holder, action);
    strictEqual(holder.run(), 3);
}
values[0] = 8;
strictEqual(values[0], 8);

const state = [3];
const increment = () => { state[0] += 1; return state[0]; };
{
    const holder: Holder = { run: () => 0 };
    install(holder, increment);
    strictEqual(holder.run(), 4);
    strictEqual(holder.run(), 5);
}
strictEqual(increment(), 6);

const inputs = [7];
const read = () => inputs[0];
const holder: Holder = { run: () => 0 };
relay(read, holder);
strictEqual(holder.run(), 7);

function installThenFail(holder: Holder, action: () => number): never {
    holder.run = action;
    throw 'installed';
}
function failBeforeInstall(holder: Holder, action: () => number, fail: boolean): void {
    if (fail) throw 'before';
    holder.run = action;
}
const failureValues = [3];
const failureAction = () => failureValues[0];
const after: Holder = { run: () => 0 };
let afterCaught = false;
try { installThenFail(after, failureAction); } catch (error) { afterCaught = error === 'installed'; }
strictEqual(afterCaught, true);
strictEqual(after.run(), 3);
const before: Holder = { run: () => 9 };
let beforeCaught = false;
try { failBeforeInstall(before, failureAction, true); } catch (error) { beforeCaught = error === 'before'; }
strictEqual(beforeCaught, true);
strictEqual(before.run(), 9);
const onceHolder = { run: () => 'initial' };
const text = 'Pat';
onceHolder.run = () => text;
strictEqual(onceHolder.run(), 'Pat');

const failurePayload = [3];
const typedAction = (fail: boolean): number => {
    if (fail) throw 'callback';
    return failurePayload[0];
};
const typedHolder = { run: (_fail: boolean): number => 0 };
typedHolder.run = typedAction;
let typedCaught = false;
try { typedHolder.run(true); } catch (error) { typedCaught = typeof error === 'string' && error === 'callback'; }
strictEqual(typedCaught, true);
strictEqual(typedHolder.run(false), 3);

function chooseFailure(flag: boolean): number {
    const holder: Holder = { run: () => 0 };
    if (flag) install(holder, () => { throw 3; });
    else install(holder, () => { throw 'four'; });
    const guarded = () => {
        try { return holder.run(); } catch (error) {
            return typeof error === 'number' ? error : typeof error === 'string' ? error.length : 0;
        }
    };
    return guarded();
}
strictEqual(chooseFailure(true), 3);
strictEqual(chooseFailure(false), 4);
