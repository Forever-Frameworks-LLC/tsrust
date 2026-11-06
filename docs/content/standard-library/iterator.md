<!-- tsrust-docs
{"id":"reference-standard-library-iterator","group":"standard-library","order":20,"status":"partial","targets":["web","native"],"api":[{"receiver":"std:iterator","name":"Iterator"},{"receiver":"std:iterator","name":"IteratorResult"},{"receiver":"IteratorConstructor","name":"from"},{"receiver":"Iterator","name":"next"},{"receiver":"Iterator","name":"map"},{"receiver":"Iterator","name":"filter"},{"receiver":"Iterator","name":"take"},{"receiver":"Iterator","name":"drop"},{"receiver":"Iterator","name":"flatMap"},{"receiver":"Iterator","name":"reduce"},{"receiver":"Iterator","name":"forEach"},{"receiver":"Iterator","name":"some"},{"receiver":"Iterator","name":"every"},{"receiver":"Iterator","name":"find"},{"receiver":"Iterator","name":"toArray"}]}
-->
# Owned lazy iterators

Import `Iterator` and `IteratorResult` from `std:iterator`. `Iterator.from(values)` consumes an owned array without cloning its elements. Arrays keep their eager methods and snapshot behavior.

```tsrust
import { Iterator } from 'std:iterator';
function main() {
  const values = Iterator.from([1, 2, 3, 4])
    .filter(value => value % 2 === 0)
    .map(value => value * 3);
  console.log(values.toArray().join(',')); // 6,12
}
```

Adaptors consume their receiver and own reusable callbacks. `next()` mutably borrows the iterator and returns `{ done: false, value }` or `{ done: true }`. Check `done` before reading `value`. No callback runs until advancement, including after `take(0)`.

Callbacks receive only the value; reduce receives accumulator and value, with a required explicit initial accumulator. map, flatMap, forEach, some and every consume their value. filter and find borrow it readonly. These differ from eager array callbacks: there are no implicit index or source arguments. flatMap accepts owned arrays and built-in iterators; arbitrary iterable protocols are outside this bundle.

A callback error propagates its existing typed error and consumes the triggering input. The iterator retains remaining inputs and captures, so another `next()` can continue. No automatic error fusion occurs. take and drop count consumed input events, including an event that throws; drop always propagates errors. Counts are nonnegative ints checked when creating the adaptor; negative counts throw a range error, without an arbitrary size cap.

reduce, forEach, some, every, find and toArray consume the chain. On an error or short circuit they drop its remainder and captures. find transfers the matched item; toArray transfers every remaining item into an owned array. Non-cloneable owners are supported. Dropping a partially consumed iterator drops its remaining owners and captures once.

Iterator shape stays concrete through owned functions, captures and records. A finite branch joining different shapes uses a finite owned carrier. Borrowed stored chains and arbitrary user iterable protocols remain outside this bundle.

Collection lowers through the owned pull environment. It does not promise to reuse the input array allocation. Standard Rust collection can specialize some static chains in place; generated output and allocation behavior are validated separately from the callback ABI admission.
