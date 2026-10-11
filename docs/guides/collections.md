# Collections

S08 is in progress. Existing `Map` and `Set` retain insertion order. The initial
`Deque<T>` surface stores an owned sequence and supports operations at both ends.
Ordered maps, ordered sets and heaps are not yet available.

```tsrust
import {Deque} from "std:collections";
function main() {
  const values = new Deque<int>();
  values.pushBack(2);
  values.pushFront(1);
  console.log(values.popFront()!, values.size); // 1 1
}
```

Construction requires an explicit element type and takes no arguments. Import
aliases work. Storage releases automatically when its owner ends.

| Member | Behavior |
| --- | --- |
| `size: int` | Readonly element count. |
| `pushFront(value)` / `pushBack(value)` | Insert one element; return void. |
| `popFront()` / `popBack()` | Remove and return an element, or undefined when empty. |
| `front()` / `back()` | Read an element without removal, or undefined when empty. |
| `at(index: int)` | Read a zero-based position; negative and out-of-range indices return undefined. |
| `clear()` | Release all stored values and set size to zero; return void. |

Insertion transfers owned values; removal transfers them back to the caller.
Scalar reads copy. Reads of owned elements borrow from a stored receiver and
prevent mutation or transfer of that receiver while the borrow remains in scope.
Use explicit undefined narrowing before using a borrowed optional element.

Byte views retain their original owners through insertion, removal and function
returns. Clearing or removing elements does not yet release their source-level
owner restrictions before the queue binding leaves scope. A readonly reborrow of
an exclusive view cannot escape the local queue that stores it.

`for...of` visits elements from front to back without removing them. Scalar
elements copy; owned elements borrow readonly for each iteration. The queue
cannot mutate or transfer while traversal borrows it. Empty queues skip the body.

Scoped callback payload integration, explicit conditional cloning and
thread-transfer capability remain unfinished. These methods do not establish
complete S08 acceptance. [The collection plan](../plans/collections-plan.md)
tracks remaining implementation and family measurements.
