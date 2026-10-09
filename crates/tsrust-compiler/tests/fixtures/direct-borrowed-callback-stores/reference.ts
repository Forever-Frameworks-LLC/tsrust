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
