# Collections

S08 is in progress. Existing `Map` and `Set` retain insertion order. The initial
`Deque<T>` surface stores an owned sequence and supports operations at both ends.
`Heap<T>` stores values for greatest-first removal. Ordered maps and ordered sets
are not yet available.

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

Records containing readonly callbacks can be stored, traversed and removed.
Different callback bodies share a checked payload representation. Borrowed captures
remain live after removal, including captured byte views. Consuming callbacks that
return an owned view can travel through a returned queue while retaining the byte
owner. A callback borrowing a local view wrapper cannot escape that wrapper.

`clone(queue)` borrows the source and creates independent queue storage. Each
stored element is deep-cloned, including owned arrays and records. Changes to the
copy do not affect the original. Cloning requires a cloneable element type even
when the queue is empty. Views, callbacks and non-cloneable resources are rejected.
Cloning costs time and storage proportional to the queue and its owned payloads.

Broader callback storage integration and thread-transfer capability remain unfinished. These methods do not establish
complete S08 acceptance. [The collection plan](../plans/collections-plan.md)
tracks remaining implementation and family measurements.

## Heap

```tsrust
import {Heap} from "std:collections";
function main() {
  const pending = new Heap<int>();
  pending.push(2);
  pending.push(9);
  console.log(pending.pop()!, pending.pop()!); // 9 2
}
```

`new Heap<T>()` takes no arguments. Its explicit element type must be int, number,
boolean, string or enum. `push(value)` transfers one value and returns void.
`peek()` reads the greatest value without removal; `pop()` removes and transfers
it. Both return undefined when empty. `size` is readonly; `clear()` releases all
values and returns void. Wrong argument counts or element types are rejected.

Duplicate values remain separate. Numbers use numeric order, signed zeros are
stored as positive zero, and all NaNs compare equal and above positive infinity.
Strings use Unicode scalar lexicographic order without locale collation or
normalization. Boolean true sorts above false; enums use their backing values.

Scalar peek results copy. String peek results borrow the stored heap; mutation
is rejected while that borrow remains live. Pop transfers owned values without
cloning. `clone(heap)` explicitly creates independent storage. `for...of` borrows every
stored element without removing it; traversal order is unspecified. Scalars copy
and strings borrow readonly. Mutation or transfer during traversal is rejected.
Repeated pop consumes values in descending order.
