type int = number;
type Holder = {
    run: () => int;
};

type Box = {
    handlers: Map<string, Holder>;
};

function workload(n: int, seed: int, input: string): int[][] {
    const output: int[][] = [];
    let i = 0;
    while (i < n) {
        const value = i % 5 + seed % 5 + 1;
        const base = [value, value + 1];
        let removedValue = 0;
        let arrayValue = 0;
        let mapFirst = 0;
        let mapSecond = 0;
        let replaced = 0;
        let iterated = 0;
        let peerValue = 0;
        let populated = 0;
        let cleared = 0;
        {
            const items: Holder[] = [{
                run: (): int => base[0]
            }];
            const removed = items.pop();
            items.push({
                run: (): int => base[1]
            });
            if (removed !== undefined) removedValue = removed.run();
            arrayValue = items[0].run();
            const handlers = new Map<string, Holder>();
            handlers.set("first", {
                run: (): int => base[0]
            });
            const peerMap = new Map<string, Holder>();
            peerMap.set("peer", { run: (): int => base[1] + 2 });
            const boxes: Box[] = [{ handlers: handlers }, { handlers: peerMap }];
            boxes[0].handlers.set("second", {
                run: (): int => base[1]
            });
            {
                const first = boxes[0].handlers.get("first");
                const second = boxes[0].handlers.get("second");
                if (first !== undefined) mapFirst = first.run();
                if (second !== undefined) mapSecond = second.run();
            }

            {
                const peer = boxes[1].handlers.get("peer");
                if (peer !== undefined) peerValue = peer.run();
            }
            populated = boxes[0].handlers.size;
            boxes[0].handlers.forEach((holder: Holder): void => {
                iterated += holder.run();
            });
            const replacementMap = new Map<string, Holder>();
            replacementMap.set("replacement", {
                run: (): int => base[1] + 1
            });
            boxes[0].handlers = replacementMap;
            {
                const replacement = boxes[0].handlers.get("replacement");
                if (replacement !== undefined) replaced = replacement.run();
            }

            boxes[0].handlers.clear();
            cleared = boxes[0].handlers.size;
        }

        const oldCapture = [value];
        const newCapture = [value + 4];
        const refillMap = new Map<string, Holder>();
        refillMap.set("old", { run: (): int => oldCapture[0] });
        refillMap.clear();
        refillMap.set("new", { run: (): int => newCapture[0] });
        oldCapture[0] = value + 5;
        let refilled = 0;
        {
            const current = refillMap.get("new");
            if (current !== undefined) refilled = current.run();
        }
        refillMap.clear();
        newCapture[0] = value + 6;
        refilled += oldCapture[0] + newCapture[0];

        base[0] = 0;
        output.push([removedValue, arrayValue, mapFirst, mapSecond, replaced, iterated, peerValue, populated, cleared, base[0] + base[1], refilled]);
        i += 1;
    }

    return output;
}

function main() {
    const result = workload(3, 0, "composition");
}
