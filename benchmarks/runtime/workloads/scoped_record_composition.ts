type int = number;
type Holder = {
    run: () => int;
};

type Box = {
    handlers: Map<string, Holder>;
};

type RefillBox = { handlers: Map<string, Holder>; marker: int; };

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
        const refillBox: RefillBox = { handlers: refillMap, marker: 1 };
        refillBox.handlers.set("old", { run: (): int => oldCapture[0] });
        if (value % 2 === 0) { refillBox.handlers.clear(); } else { refillBox.handlers.clear(); }
        refillBox.handlers.set("new", { run: (): int => newCapture[0] });
        oldCapture[0] = value + 5;
        let refilled = 0;
        {
            const current = refillBox.handlers.get("new");
            if (current !== undefined) refilled = current.run();
        }
        let movedSize = 0;
        if (value % 2 === 0) { refillBox.handlers.clear(); const moved = refillBox.handlers; movedSize = moved.size; } else { refillBox.handlers.clear(); movedSize = refillBox.handlers.size; }

        newCapture[0] = value + 6;
        refilled += oldCapture[0] + newCapture[0] + refillBox.marker;

        const beforeThrow = [value];
        const afterThrow = [value + 8];
        const caughtMap = new Map<string, Holder>();
        caughtMap.set("old", { run: (): int => beforeThrow[0] });
        let caughtValue = 0;
        try {
            caughtMap.clear();
            caughtMap.set("new", { run: (): int => afterThrow[0] });
            throw value;
        } catch (error) {
            caughtValue = caughtMap.size;
        }
        beforeThrow[0] = value + 9;
        {
            const current = caughtMap.get("new");
            if (current !== undefined) caughtValue += current.run();
        }
        caughtMap.clear();
        afterThrow[0] = value + 10;
        caughtValue += beforeThrow[0] + afterThrow[0];

        let branchTrace = 0;
        let normalCleanup = 0;
        let caughtCleanup = 0;
        let normalRenewed = 0;
        let caughtRenewed = 0;
        {
            const prefixOld = [value];
            const prefixNew = [value + 12];
            const prefixMap = new Map<string, Holder>();
            prefixMap.set("old", { run: (): int => prefixOld[0] });
            let prefixValue = 0;
            try { if (value % 2 === 0) { branchTrace += 1;
                prefixValue = prefixResult(false);
                prefixMap.clear();
                prefixMap.set("new", { run: (): int => prefixNew[0] });
            } else { branchTrace += 2;
                prefixValue = prefixResult(false);
                prefixMap.clear();
                prefixMap.set("new", { run: (): int => prefixNew[0] });
            } } catch (error) { {
                prefixValue = 2;
                prefixValue += prefixMap.size;
                prefixMap.clear();
                prefixMap.set("new", { run: (): int => prefixNew[0] });
            } } finally { normalCleanup = prefixMap.size; }
            prefixOld[0] = value + 13;
            {
                const entry = prefixMap.get("new");
                if (entry !== undefined) prefixValue += entry.run();
            }
            prefixMap.clear();
            prefixNew[0] = value + 14;
            normalRenewed = prefixValue + prefixOld[0] + prefixNew[0];
        }
        {
            const prefixOld = [value];
            const prefixNew = [value + 12];
            const prefixMap = new Map<string, Holder>();
            prefixMap.set("old", { run: (): int => prefixOld[0] });
            let prefixValue = 0;
            try { if (value % 2 === 0) { branchTrace += 1;
                prefixValue = prefixResult(true);
                prefixMap.clear();
                prefixMap.set("new", { run: (): int => prefixNew[0] });
            } else { branchTrace += 2;
                prefixValue = prefixResult(true);
                prefixMap.clear();
                prefixMap.set("new", { run: (): int => prefixNew[0] });
            } } catch (error) { {
                prefixValue = 2;
                prefixValue += prefixMap.size;
                prefixMap.clear();
                prefixMap.set("new", { run: (): int => prefixNew[0] });
            } } finally { caughtCleanup = prefixMap.size; }
            prefixOld[0] = value + 13;
            {
                const entry = prefixMap.get("new");
                if (entry !== undefined) prefixValue += entry.run();
            }
            prefixMap.clear();
            prefixNew[0] = value + 14;
            caughtRenewed = prefixValue + prefixOld[0] + prefixNew[0];
        }

        let normalConsumed = 0;
        let caughtConsumed = 0;
        {
            const consumedOld = [value];
            const consumedMap = new Map<string, Holder>();
            consumedMap.set("old", { run: (): int => consumedOld[0] });
            let consumedPath = 0;
            let consumedSize = -1;
            try {
                consumedPath = prefixResult(false);
                consumedMap.clear();
                const moved = consumedMap;
                consumedSize = moved.size;
            } catch (error) {
                consumedPath = 2;
                consumedMap.clear();
                const moved = consumedMap;
                consumedSize = moved.size;
            }
            consumedOld[0] = value + 20;
            normalConsumed = consumedSize + consumedPath + consumedOld[0];
        }
        {
            const consumedOld = [value];
            const consumedMap = new Map<string, Holder>();
            consumedMap.set("old", { run: (): int => consumedOld[0] });
            let consumedPath = 0;
            let consumedSize = -1;
            try {
                consumedPath = prefixResult(true);
                consumedMap.clear();
                const moved = consumedMap;
                consumedSize = moved.size;
            } catch (error) {
                consumedPath = 2;
                consumedMap.clear();
                const moved = consumedMap;
                consumedSize = moved.size;
            }
            consumedOld[0] = value + 20;
            caughtConsumed = consumedSize + consumedPath + consumedOld[0];
        }

        let earlyRestored = 0;
        let lateRestored = 0;
        {
            const restoredOld = [value];
            let restoredMap = new Map<string, Holder>();
            restoredMap.set("old", { run: (): number => restoredOld[0] });
            let restoredSize = -1;
            try {
                const beforeTransfer = prefixResult(true);
                restoredMap.clear();
                const transferred = restoredMap;
                restoredSize = transferred.size;
                throw 7;
            } catch (error) {
                restoredMap = new Map<string, Holder>();
            }
            restoredOld[0] = value + 30;
            earlyRestored = restoredSize + restoredMap.size + restoredOld[0];
        }

        {
            const restoredOld = [value];
            let restoredMap = new Map<string, Holder>();
            restoredMap.set("old", { run: (): number => restoredOld[0] });
            let restoredSize = -1;
            try {
                const beforeTransfer = prefixResult(false);
                restoredMap.clear();
                const transferred = restoredMap;
                restoredSize = transferred.size;
                throw 7;
            } catch (error) {
                restoredMap = new Map<string, Holder>();
            }
            restoredOld[0] = value + 30;
            lateRestored = restoredSize + restoredMap.size + restoredOld[0];
        }

        let earlyExpression = 0;
        let lateExpression = 0;
        {
            const expressionOld = [value];
            let expressionMap = new Map<string, Holder>();
            expressionMap.set("old", { run: (): number => expressionOld[0] });
            let expressionSaved = new Map<string, Holder>();
            let expressionResult = -1;
            try {
                expressionMap.clear();
                expressionResult = (prefixResult(true), expressionSaved = expressionMap, 1);
                throw 8;
            } catch (error) {
                expressionMap = new Map<string, Holder>();
            }
            expressionOld[0] = value + 40;
            earlyExpression = expressionResult + expressionSaved.size + expressionMap.size + expressionOld[0];
        }

        {
            const expressionOld = [value];
            let expressionMap = new Map<string, Holder>();
            expressionMap.set("old", { run: (): number => expressionOld[0] });
            let expressionSaved = new Map<string, Holder>();
            let expressionResult = -1;
            try {
                expressionMap.clear();
                expressionResult = (expressionSaved = expressionMap, prefixResult(true));
                throw 8;
            } catch (error) {
                expressionMap = new Map<string, Holder>();
            }
            expressionOld[0] = value + 41;
            lateExpression = expressionResult + expressionSaved.size + expressionMap.size + expressionOld[0];
        }

        let conditionalRenewed = 0;
        {
            const conditionalOld = [value];
            const conditionalMap = new Map<string, Holder>();
            conditionalMap.set("old", { run: (): number => conditionalOld[0] });
            const conditionalBox: Box = { handlers: conditionalMap };
            let conditionalSize = -1;
            try {
                if (value % 2 === 0) {
                    conditionalBox.handlers.clear();
                    const conditionalMoved = conditionalBox.handlers;
                    conditionalSize = conditionalMoved.size;
                } else {
                    conditionalSize = conditionalBox.handlers.size;
                }
                throw 9;
            } catch (error) {
                conditionalBox.handlers = new Map<string, Holder>();
            }
            conditionalOld[0] = value + 50;
            conditionalRenewed = conditionalSize + conditionalBox.handlers.size + conditionalOld[0];
        }

        let normalSurvivor = 0;
        const survivorOld = [value];
        {
            let survivorMap = new Map<string, Holder>();
            survivorMap.set("old", { run: (): number => survivorOld[0] });
            let survivorSize = -1;
            try {
                const survivorPrefix = prefixResult(false);
                if (value % 2 === 0) {
                    survivorMap.clear();
                    const survivorMoved = survivorMap;
                    survivorSize = survivorMoved.size;
                } else {
                    survivorSize = survivorMap.size;
                }
            } catch (error) {
                survivorMap = new Map<string, Holder>();
            }
            normalSurvivor = survivorSize + survivorOld[0];
        }
        survivorOld[0] = value + 51;
        normalSurvivor += survivorOld[0];

        let outsideRestored = 0;
        {
            const outsideOld = [value];
            let outsideMap = new Map<string, Holder>();
            outsideMap.set("old", { run: (): number => outsideOld[0] });
            let outsideSize = -1;
            if (value % 2 === 0) {
                outsideMap.clear();
                const outsideMoved = outsideMap;
                outsideSize = outsideMoved.size;
            } else {
                outsideSize = outsideMap.size;
            }
            outsideMap = new Map<string, Holder>();
            outsideOld[0] = value + 60;
            outsideRestored = outsideSize + outsideMap.size + outsideOld[0];
        }

        const cleanupTrace = [0, -1];
        let finallyRenewed = 0;
        try {
            finallyRenewed = noCatchRenewal(value % 2 === 0, value, cleanupTrace);
        } catch (error) {
            finallyRenewed = 7;
        }
        finallyRenewed += cleanupTrace[0] * 100 + cleanupTrace[1];

        const returnTrace = [0, -1];
        const finallyReturned = noCatchReturn(value % 2 === 0, value, returnTrace) + returnTrace[0] * 100 + returnTrace[1];
        const loopTrace = [0, 0];
        const finallyLooped = noCatchLoop(value, loopTrace) + loopTrace[0] * 100 + loopTrace[1];
        const outerTrace = [0, 0];
        const finallyOuterLooped = noCatchOuterLoop(value, outerTrace) + outerTrace[0] * 100 + outerTrace[1];
        const initializedRecordMap = nestedMapInitializer(value);
        const recordLoopTrace = [0, 0];
        const loopedRecordMap = recordMapLoop(value, recordLoopTrace) + recordLoopTrace[0] * 100 + recordLoopTrace[1];
        const nestedTrace = [0, 0, 0, 0];
        let nestedCleaned = 0;
        try {
            nestedCleaned = nestedMapCleanup(value % 2 === 0, value, nestedTrace);
        } catch (error) {
            nestedCleaned = 7;
        }
        nestedCleaned += nestedTrace[0] * 100 + nestedTrace[1] * 10 + nestedTrace[2] * 1000 + nestedTrace[3];
        base[0] = 0;
        const caughtNestedTrace = [0, 0, 0, 0];
        const nestedCaught = nestedCaughtMapCleanup(value % 2 === 0, value, caughtNestedTrace) + caughtNestedTrace[0] * 100 + caughtNestedTrace[1] * 10 + caughtNestedTrace[2] * 1000 + caughtNestedTrace[3];
        const abruptTrace=[0,0,0,0];
        let nestedAbrupt=0;
        try {nestedAbrupt=nestedAbruptCleanup(value % 2 === 0,value,abruptTrace);}catch(error){nestedAbrupt=11;}
        nestedAbrupt+=abruptTrace[0]*100+abruptTrace[1]*10+abruptTrace[2]*1000+abruptTrace[3];
        const returnedTrace=[0,0,0];
        const nestedReturned=nestedReturnedCleanup(value % 2 === 0,value,returnedTrace)+returnedTrace[0]*100+returnedTrace[1]*10+returnedTrace[2]*1000;
        const caughtLoopTrace=[0,0];
        const caughtLooped=caughtMapLoop(value,caughtLoopTrace)+caughtLoopTrace[0]*100+caughtLoopTrace[1];
        const pairedLoopTrace = [0, 0]; const pairedLooped = pairedMapLoop(value, pairedLoopTrace) + pairedLoopTrace[0] * 100 + pairedLoopTrace[1];
        const nestedPairTrace=[0,0];const nestedPairLooped=nestedPairedMapLoop(value,nestedPairTrace)+nestedPairTrace[0]*100+nestedPairTrace[1];
        const forOfMapObserved=forOfMapLoop();
        const countedMapObserved=countedMapLoop();const postMapObserved=postMapLoop();
        const emptyMapObserved=emptyScopedMap();
        output.push([removedValue, arrayValue, mapFirst, mapSecond, replaced, iterated, peerValue, populated, cleared, base[0] + base[1], refilled, movedSize, caughtValue, normalRenewed, caughtRenewed, normalCleanup, caughtCleanup, branchTrace, normalConsumed, caughtConsumed, earlyRestored, lateRestored, earlyExpression, lateExpression, conditionalRenewed, normalSurvivor, outsideRestored, finallyRenewed, finallyReturned, finallyLooped, finallyOuterLooped, initializedRecordMap, loopedRecordMap, nestedCleaned, nestedCaught, nestedAbrupt, nestedReturned, caughtLooped, pairedLooped, nestedPairLooped, emptyMapObserved, countedMapObserved, postMapObserved, forOfMapObserved, knownEntryRelease(value), movedFieldEntryRelease(value), replacementEntryOutcome(value,value%2!==0), branchEntryRelease(value, value % 2 === 0), mixedBranchEntryRelease(value, value % 2 === 0), pairedBranchEntryRelease(value, value % 2 === 0)]);
        i += 1;
    }

    return output;
}

function prefixResult(fail: boolean): int {
    if (fail) throw 5;
    return 1;
}

function main() {
    const result = workload(3, 0, "composition");
}

function noCatchRenewal(fail: boolean, value: number, trace: number[]): number {
    const finallyOld = [value];
    const finallyNext = [value + 1];
    let finallyMap = new Map<string, Holder>();
    finallyMap.set("old", { run: (): number => finallyOld[0] });
    let retained = 0;
    try {
        const finallyPrefix = prefixResult(fail);
        finallyMap.clear();
        finallyMap.set("next", { run: (): number => finallyNext[0] });
        retained = finallyMap.size;
    } finally {
        trace[0] = finallyMap.size;
        finallyMap.clear();
        trace[1] = finallyMap.size;
    }
    finallyOld[0] = value + 70;
    finallyNext[0] = value + 71;
    return retained + finallyOld[0] + finallyNext[0];
}

function noCatchReturn(early: boolean, value: number, trace: number[]): number {
    const returnOld = [value];
    const returnNext = [value + 1];
    let returnMap = new Map<string, Holder>();
    returnMap.set("old", { run: (): number => returnOld[0] });
    try {
        const returnPrefix = prefixResult(false);
        if (early) return value + 80;
        returnMap.clear();
        returnMap.set("next", { run: (): number => returnNext[0] });
    } finally {
        trace[0] = returnMap.size;
        returnMap.clear();
        trace[1] = returnMap.size;
    }
    returnOld[0] = value + 82;
    returnNext[0] = value + 83;
    return returnOld[0] + returnNext[0];
}

function noCatchLoop(value: number, trace: number[]): number {
    const loopOld = [value];
    const loopNext = [value + 1];
    let turn = 0;
    let loopObserved = 0;
    while (turn < 3) {
        let loopMap = new Map<string, Holder>();
        loopMap.set("old", { run: (): number => loopOld[0] });
        turn += 1;
        try {
            const loopPrefix = prefixResult(false);
            loopMap.clear();
            loopMap.set("next", { run: (): number => loopNext[0] });
            if (turn === 1) continue;
            if (turn === 3) break;
        } finally {
            trace[0] += loopMap.size;
            loopMap.clear();
            trace[1] += loopMap.size;
        }
        loopOld[0] = value + 90;
        loopNext[0] = value + 91;
        loopObserved = loopOld[0] + loopNext[0];
    }
    return loopObserved + turn;
}

function noCatchOuterLoop(value: number, trace: number[]): number {
    const loopOld = [value];
    const loopNext = [value + 1];
    let turn = 0;
    let loopObserved = 0;
    let loopMap = new Map<string, Holder>();
    while (turn < 3) {
        loopMap.set("old", { run: (): number => loopOld[0] });
        turn += 1;
        try {
            const loopPrefix = prefixResult(false);
            loopMap.clear();
            loopMap.set("next", { run: (): number => loopNext[0] });
            if (turn === 1) continue;
            if (turn === 3) break;
        } finally {
            trace[0] += loopMap.size;
            loopMap.clear();
            trace[1] += loopMap.size;
        }
        loopOld[0] = value + 90;
        loopNext[0] = value + 91;
        loopObserved = loopOld[0] + loopNext[0];
    }
    return loopObserved + turn + loopMap.size;
}

function nestedMapInitializer(value: number): number {
    const initializerOld = [value];
    const initializerNext = [value + 1];
    const initializerBox: RefillBox = { marker: value + 100, handlers: new Map<string, Holder>() };
    initializerBox.handlers.set("old", { run: (): number => initializerOld[0] });
    initializerBox.handlers.set("next", { run: (): number => initializerNext[0] });
    const initializerObserved = initializerBox.handlers.size + initializerBox.marker;
    initializerBox.handlers.clear();
    initializerOld[0] = value + 101;
    initializerNext[0] = value + 102;
    return initializerObserved + initializerOld[0] + initializerNext[0] + initializerBox.handlers.size;
}

function recordMapLoop(value: number, trace: number[]): number {
    const recordLoopOld = [value];
    const recordLoopNext = [value + 1];
    let turn = 0;
    let recordLoopObserved = 0;
    let recordLoopBox: RefillBox = { marker: value + 110, handlers: new Map<string, Holder>() };
    while (turn < 3) {
        recordLoopBox.handlers.set("old", { run: (): number => recordLoopOld[0] });
        turn += 1;
        try {
            const recordLoopPrefix = prefixResult(false);
            recordLoopBox.handlers.clear();
            recordLoopBox.handlers.set("next", { run: (): number => recordLoopNext[0] });
            if (turn === 1) continue;
            if (turn === 3) break;
        } finally {
            trace[0] += recordLoopBox.handlers.size;
            recordLoopBox.handlers.clear();
            trace[1] += recordLoopBox.handlers.size;
        }
        recordLoopBox.marker += 1;
        recordLoopOld[0] = value + 111;
        recordLoopNext[0] = value + 112;
        recordLoopObserved = recordLoopOld[0] + recordLoopNext[0] + recordLoopBox.marker;
    }
    const recordLoopOrder = Object.keys(recordLoopBox)[0] === "marker" ? 4 : 0;
    return recordLoopObserved + turn + recordLoopBox.handlers.size + recordLoopOrder;
}

function nestedCaughtMapCleanup(fail: boolean, value: number, trace: number[]): number {
    const caughtNestedOld = [value];
    const caughtNestedNext = [value + 1];
    let caughtNestedMap = new Map<string, Holder>();
    caughtNestedMap.set("old", { run: (): number => caughtNestedOld[0] });
    let caughtNestedObserved = -1;
    try {
        try {
            const caughtNestedPrefix = prefixResult(fail);
            caughtNestedMap.clear();
            caughtNestedMap.set("next", { run: (): number => caughtNestedNext[0] });
            caughtNestedObserved = caughtNestedMap.size;
        } catch (error) {
            caughtNestedObserved = 7;
            caughtNestedMap.clear();
            caughtNestedMap.set("next", {run: () => caughtNestedNext[0]});
            trace[0] += caughtNestedMap.size;
            caughtNestedMap.clear();
            trace[1] += caughtNestedMap.size;
        }
    } finally {
        trace[2] += caughtNestedMap.size;
        caughtNestedMap.clear();
        trace[3] += caughtNestedMap.size;
    }
    caughtNestedOld[0] = value + 120;
    caughtNestedNext[0] = value + 121;
    return caughtNestedObserved + caughtNestedOld[0] + caughtNestedNext[0];
}

function nestedMapCleanup(fail: boolean, value: number, trace: number[]): number {
    const nestedOld = [value];
    const nestedNext = [value + 1];
    let nestedMap = new Map<string, Holder>();
    nestedMap.set("old", { run: (): number => nestedOld[0] });
    let nestedObserved = -1;
    try {
        try {
            const nestedPrefix = prefixResult(fail);
            nestedMap.clear();
            nestedMap.set("next", { run: (): number => nestedNext[0] });
            nestedObserved = nestedMap.size;
        } finally {
            trace[0] += nestedMap.size;
            nestedMap.clear();
            trace[1] += nestedMap.size;
        }
    } finally {
        trace[2] += nestedMap.size;
        nestedMap.clear();
        trace[3] += nestedMap.size;
    }
    nestedOld[0] = value + 120;
    nestedNext[0] = value + 121;
    return nestedObserved + nestedOld[0] + nestedNext[0];
}

function nestedAbruptCleanup(fail: boolean, value: number, trace: number[]): number {
 const abruptOld=[value]; const abruptNext=[value+1];
 let abruptMap=new Map<string,Holder>();
 abruptMap.set("old",{run:():number=>abruptOld[0]});
 let abruptObserved=-1;
 try {
  try {const abruptPrefix=prefixResult(fail);abruptMap.clear();abruptMap.set("next",{run:():number=>abruptNext[0]});abruptObserved=abruptMap.size;}
  catch(error){abruptMap.clear();abruptMap.set("next",{run:():number=>abruptNext[0]});throw 9;}
  finally {trace[0]+=abruptMap.size;abruptMap.clear();trace[1]+=abruptMap.size;if(fail)throw 11;}
 } finally {trace[2]+=abruptMap.size;abruptMap.clear();trace[3]+=abruptMap.size;}
 abruptOld[0]=value+120;abruptNext[0]=value+121;
 return abruptObserved+abruptOld[0]+abruptNext[0];
}

function nestedReturnedCleanup(fail: boolean, value: number, trace: number[]): number {
 const returnedOld=[value];const returnedNext=[value+1];
 let returnedMap=new Map<string,Holder>();returnedMap.set("old",{run:():number=>returnedOld[0]});
 try {
  try {const returnedPrefix=prefixResult(fail);returnedMap.clear();returnedMap.set("next",{run:():number=>returnedNext[0]});return returnedOld[0]+value;}
  catch(error){returnedMap.clear();returnedMap.set("next",{run:():number=>returnedNext[0]});return returnedNext[0]+value;}
  finally {trace[0]+=returnedMap.size;returnedMap.clear();trace[1]+=returnedMap.size;}
 } finally {returnedMap.clear();returnedOld[0]=0;returnedNext[0]=0;trace[2]+=returnedMap.size;}
}

function caughtMapLoop(value: number, trace: number[]): number {
 const caughtLoopOld=[value];const caughtLoopNext=[value+1];let turn=0;let caughtLoopObserved=0;
 let caughtLoopMap=new Map<string,Holder>();
 while(turn<3){
  caughtLoopMap.set("old",{run:():number=>caughtLoopOld[0]});turn+=1;
  try{const caughtLoopPrefix=prefixResult(turn!==2);caughtLoopMap.clear();caughtLoopMap.set("next",{run:():number=>caughtLoopNext[0]});}
  catch(error){caughtLoopMap.clear();caughtLoopMap.set("next",{run:():number=>caughtLoopNext[0]});if(turn===1)continue;break;}
  finally{trace[0]+=caughtLoopMap.size;caughtLoopMap.clear();trace[1]+=caughtLoopMap.size;caughtLoopOld[0]=value+140;caughtLoopNext[0]=value+141;}
  caughtLoopObserved=caughtLoopOld[0]+caughtLoopNext[0];
 }
 caughtLoopOld[0]=value+142;caughtLoopNext[0]=value+143;
 return caughtLoopObserved+turn+caughtLoopMap.size+caughtLoopOld[0]+caughtLoopNext[0];
}

function pairedMapLoop(value: number, trace: number[]): number {
 const pairedOld=[value];const pairedNext=[value+1];let turn=0;
 const spareMap=new Map<string,int>();
 let pairedLeft=new Map<string,Holder>();let pairedRight=new Map<string,Holder>();
 while(turn<3){
  pairedLeft.set("old",{run:():number=>pairedOld[0]});pairedRight.set("next",{run:():number=>pairedNext[0]});turn+=1;
  try{const pairedPrefix=prefixResult(turn!==2);}
  catch(error){if(turn===1)continue;break;}
  finally{trace[0]+=pairedLeft.size+pairedRight.size;pairedLeft.clear();pairedRight.clear();pairedOld[0]=value+150;pairedNext[0]=value+151;trace[1]+=pairedLeft.size+pairedRight.size;}
 }
 trace[1]+=spareMap.size;
 return pairedOld[0]+pairedNext[0]+turn+pairedLeft.size+pairedRight.size;
}

function nestedPairedMapLoop(value: number, trace: number[]): number {
 const nestedPairOld=[value];const nestedPairNext=[value+1];let turn=0;
 let nestedPairLeft=new Map<string,Holder>();let nestedPairRight=new Map<string,Holder>();
 while(turn<3){
  nestedPairLeft.set("old",{run:():number=>nestedPairOld[0]});nestedPairRight.set("next",{run:():number=>nestedPairNext[0]});turn+=1;
  try{try{const nestedPairPrefix=prefixResult(turn!==2);}
  catch(error){if(turn===1)continue;break;}
  finally{trace[1]+=turn;}}catch(error){trace[1]+=1000;}finally{trace[0]+=nestedPairLeft.size+nestedPairRight.size;nestedPairLeft.clear();nestedPairRight.clear();nestedPairOld[0]=value+150;nestedPairNext[0]=value+151;trace[1]+=nestedPairLeft.size+nestedPairRight.size;}
 }
 return nestedPairOld[0]+nestedPairNext[0]+turn+nestedPairLeft.size+nestedPairRight.size;
}

function emptyScopedMap(): number {
 const unfilled=new Map<string,Holder>();let emptyObserved=unfilled.size;
 if(unfilled.has("none"))emptyObserved+=100;
 if(unfilled.delete("none"))emptyObserved+=1000;
 {const absent=unfilled.get("none");emptyObserved+=absent===undefined?1:0;if(absent!==undefined)emptyObserved+=absent.run();}
 for(const [key,entry] of unfilled){emptyObserved+=entry.run();}
 const emptyMoved=unfilled;emptyObserved+=emptyMoved.size;emptyMoved.clear();return emptyObserved;
}

function countedMapLoop(): number {
 const countedValues=[3];let countedTurn=0;let countedHandlers=new Map<string,Holder>();
 for(let step=0;step<3;step+=1){
  countedHandlers.set("value",{run:():number=>countedValues[0]});
  try{prefixResult(countedTurn!==1);}catch(error){if(countedTurn===0)continue;break;}
  finally{countedHandlers.clear();countedValues[0]+=2;countedTurn+=1;}
 }
 return countedTurn+countedHandlers.size+countedValues[0];
}
function postMapLoop(): number {
 const postValues=[3];let postTurn=0;let postHandlers=new Map<string,Holder>();
 do{
  postHandlers.set("value",{run:():number=>postValues[0]});
  try{prefixResult(postTurn!==1);}catch(error){if(postTurn===0)continue;break;}
  finally{postHandlers.clear();postValues[0]+=2;postTurn+=1;}
 }while(postTurn<3);
 return postTurn+postHandlers.size+postValues[0];
}

function forOfMapLoop(): number {
 const iterValues=[3];const iterSteps=[0,1,2];let iterHandlers=new Map<string,Holder>();
 for(const step of iterSteps){
  iterHandlers.set("value",{run:():number=>iterValues[0]});
  try{prefixResult(step!==1);}catch(error){if(step===0)continue;break;}
  finally{iterHandlers.clear();iterValues[0]+=2;}
 }
 return iterHandlers.size+iterValues[0];
}

function knownEntryRelease(value:number):number {
 const releaseFirst=[value];const releaseSecond=[value+1];const releaseMap=new Map<string,Holder>();
 releaseMap.set("first",{run:():number=>releaseFirst[0]});releaseMap.set("second",{run:():number=>releaseSecond[0]});
 let releaseObserved=releaseMap.delete("first")?1:0;releaseFirst[0]=value+2;
 {const releaseSaved=releaseMap.get("second");if(releaseSaved!==undefined)releaseObserved+=releaseSaved.run();}
 releaseObserved+=releaseFirst[0]+releaseMap.size;releaseMap.clear();releaseSecond[0]=value+3;
 return releaseObserved+releaseSecond[0];
}
type ReleaseInner={handlers:Map<string,Holder>;label:string;};
type ReleaseOuter={inner:ReleaseInner;marker:number;};
function movedFieldEntryRelease(value:number):number {
 const fieldFirst=[value];const fieldSecond=[value+1];
 const fieldOriginal:ReleaseOuter={inner:{handlers:new Map<string,Holder>(),label:"kept"},marker:23};const fieldMoved=fieldOriginal;
 fieldMoved.inner.handlers.set("first",{run:():number=>fieldFirst[0]});fieldMoved.inner.handlers.set("second",{run:():number=>fieldSecond[0]});
 let fieldObserved=fieldMoved.inner.handlers.delete("first")?1:0;fieldFirst[0]=value+2;
 fieldMoved.inner.handlers.set("next",{run:():number=>fieldFirst[0]});
 for(const [key,entry] of fieldMoved.inner.handlers){fieldObserved+=entry.run();}
 fieldObserved+=fieldMoved.inner.label.length+fieldMoved.marker;fieldMoved.inner.handlers.clear();fieldFirst[0]=value+4;fieldSecond[0]=value+5;
 return fieldObserved+fieldFirst[0]+fieldSecond[0];
}
function replacementEntryOutcome(value:number,fail:boolean):number {
 const outcomeFirst=[value];const outcomeNext=[value+1];const outcomeMap=new Map<string,Holder>();
 outcomeMap.set("first",{run:():number=>outcomeFirst[0]});let outcomeObserved=0;
 try{outcomeMap.set("first",(prefixResult(fail),{run:():number=>outcomeNext[0]}));}catch(error){outcomeObserved=5;}
 {const outcomeSaved=outcomeMap.get("first");if(outcomeSaved!==undefined)outcomeObserved+=outcomeSaved.run();}
 outcomeMap.clear();outcomeFirst[0]=value+4;outcomeNext[0]=value+5;
 return outcomeObserved+outcomeFirst[0]+outcomeNext[0];
}

function branchEntryRelease(value: number, branch: boolean): number {
    const branchFirst = [value];
    const branchSecond = [value + 1];
    const branchHandlers = new Map<string, Holder>();
    branchHandlers.set("first", { run: (): number => branchFirst[0] });
    branchHandlers.set("second", { run: (): number => branchSecond[0] });
    let branchObserved = 0;
    if (branch) {
        branchHandlers.delete("first");
        branchObserved = 1;
    } else {
        branchHandlers.delete("first");
        branchObserved = 2;
    }
    branchFirst[0] = value + 2;
    for (const [key, entry] of branchHandlers) { branchObserved += entry.run(); }
    branchObserved += branchFirst[0] + branchHandlers.size;
    branchHandlers.clear();
    branchSecond[0] = value + 3;
    return branchObserved + branchSecond[0];
}

function mixedBranchEntryRelease(value: number, branch: boolean): number {
    const mixedFirst = [value];
    const mixedSecond = [value + 1];
    const mixedHandlers = new Map<string, Holder>();
    mixedHandlers.set("first", { run: (): number => mixedFirst[0] });
    mixedHandlers.set("second", { run: (): number => mixedSecond[0] });
    if (branch) { mixedHandlers.clear(); } else { mixedHandlers.delete("first"); }
    mixedFirst[0] = value + 2;
    let mixedObserved = mixedHandlers.size + mixedFirst[0];
    for (const [key, entry] of mixedHandlers) { mixedObserved += entry.run(); }
    mixedHandlers.clear();
    mixedSecond[0] = value + 3;
    return mixedObserved + mixedSecond[0];
}

function pairedBranchEntryRelease(value: number, branch: boolean): number {
    const pairedFirst = [value];
    const pairedSecond = [value + 1];
    const pairedThird = [value + 2];
    const pairedLeft = new Map<string, Holder>();
    const pairedRight = new Map<string, Holder>();
    pairedLeft.set("first", { run: (): number => pairedFirst[0] });
    pairedLeft.set("second", { run: (): number => pairedSecond[0] });
    pairedRight.set("first", { run: (): number => pairedFirst[0] });
    pairedRight.set("third", { run: (): number => pairedThird[0] });
    if (branch) { pairedLeft.delete("first"); pairedRight.delete("first"); }
    else { pairedRight.delete("first"); pairedLeft.delete("first"); }
    pairedFirst[0] = value + 3;
    let pairedObserved = pairedFirst[0] + pairedLeft.size + pairedRight.size;
    for (const [key, entry] of pairedLeft) { pairedObserved += entry.run(); }
    for (const [key, entry] of pairedRight) { pairedObserved += entry.run(); }
    pairedLeft.clear(); pairedRight.clear();
    pairedSecond[0] = value + 4; pairedThird[0] = value + 5;
    return pairedObserved + pairedSecond[0] + pairedThird[0];
}
