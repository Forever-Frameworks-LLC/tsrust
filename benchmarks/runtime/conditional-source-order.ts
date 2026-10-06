type int = number;
type Trace = { tag: int; };
type Choice = { tag: int; payload: int[]; };
function predicate(trace: Trace, flag: boolean): boolean { trace.tag = trace.tag * 10 + 1; return flag; }
function left(trace: Trace, value: int): int { trace.tag = trace.tag * 10 + 2; return value; }
function right(trace: Trace, value: int): int { trace.tag = trace.tag * 10 + 3; return value; }
function skipped(trace: Trace): int { trace.tag = trace.tag * 10 + 4; const empty: int[] = []; return empty[0]; }
function main() {
    const trace: Trace = { tag: 0 };
    const first = predicate(trace, true) ? left(trace, 7) : right(trace, 9);
    console.log(first); console.log(trace.tag);
    const second = predicate(trace, false) ? left(trace, 11) : right(trace, 13);
    console.log(second); console.log(trace.tag);
    const safe = predicate(trace, true) ? left(trace, 17) : skipped(trace);
    console.log(safe); console.log(trace.tag);
    let initial: int = 2; let other: int = 3;
    const assigned = false ? (initial = 99) : (other = 7);
    console.log(assigned); console.log(initial); console.log(other);
    const nested = false ? 1 : true ? 2 : 3;
    console.log(nested);
    const ownedLeft = [11, 12]; const ownedRight = [21, 22, 23];
    const selectedOwner: int[] = true ? ownedLeft : ownedRight;
    console.log(selectedOwner[0]); console.log(selectedOwner[1]); console.log(selectedOwner.length);
    const leftRecord: Choice = { tag: 31, payload: [32, 33] };
    const rightRecord: Choice = { tag: 41, payload: [42, 43, 44] };
    const selectedRecord: Choice = false ? leftRecord : rightRecord;
    console.log(selectedRecord.tag); console.log(selectedRecord.payload[0]); console.log(selectedRecord.payload[1]); console.log(selectedRecord.payload[2]); console.log(selectedRecord.payload.length);
    const ownedLeftText = "left".repeat(2); const ownedRightText = "right".repeat(2);
    const selectedText: string = false ? ownedLeftText : ownedRightText;
    console.log(selectedText);
    const empty: int[] = true ? [] : [1]; console.log(empty.length);
    const contextual: Choice = true ? { tag: 51, payload: [] } : { tag: 52, payload: [53] };
    console.log(contextual.tag); console.log(contextual.payload.length);
}
