<!-- tsrust-docs
{"id":"reference-standard-library-logging","group":"standard-library","order":21,"status":"partial","targets":["native"],"api":[{"receiver":"std:log","name":"Logger"},{"receiver":"std:log","name":"LoggerOptions"},{"receiver":"std:log","name":"LogError"},{"receiver":"std:log","name":"LogLevel"},{"receiver":"std:log","name":"createLogger"},{"receiver":"std:log","name":"debug"},{"receiver":"std:log","name":"info"},{"receiver":"std:log","name":"warn"},{"receiver":"std:log","name":"error"},{"receiver":"std:log","name":"child"},{"receiver":"std:log","name":"flush"},{"receiver":"std:log","name":"close"},{"receiver":"Logger","name":"debug"},{"receiver":"Logger","name":"info"},{"receiver":"Logger","name":"warn"},{"receiver":"Logger","name":"error"},{"receiver":"Logger","name":"child"},{"receiver":"Logger","name":"flush"},{"receiver":"Logger","name":"close"},{"receiver":"LoggerOptions","name":"sink"},{"receiver":"LoggerOptions","name":"path"},{"receiver":"LoggerOptions","name":"level"},{"receiver":"LogError","name":"code"},{"receiver":"LogError","name":"message"}]}
-->
# Structured logging


Create a logger with an explicit stdout, stderr or append-only file sink. File sinks require path; console sinks reject path. Level defaults to info. Backend JSON timestamps, target and span ancestry remain enabled. This installs no global subscriber and leaves console unchanged.

```tsrust
import {createLogger} from 'std:log';
function main(){
  const root=unwrap(createLogger({sink:'stdout'}));
  const logger=root.child({service:'api',worker:7});
  logger.info(()=>({message:'ready',ok:true,count:3}));
  unwrap(logger.close());
  unwrap(root.close());
}
```

The INFO event retains numeric count and boolean ok, with contextual service/worker fields in backend span records. Timestamps and target values vary. Child records move once; immutable configuration and parent context are shared. A child created before a request enters that request ancestry when it emits, not when it was constructed.

The zero-argument factory runs once only when its level is enabled. It returns a concrete record with a string message and string/int/number/boolean fields. Captures are still constructed eagerly and move into the callback; a disabled callback is dropped without running. Factory errors propagate through ordinary checked calls. Arrays, records as field values, native owners, callbacks, optional fields, explicit null and undefined fields are rejected; the backend has no matching typed scalar Value for them. Nonfinite number values follow backend JSON null behavior.

For stored calls, import a free function, for example `const write = info`, and
pass the logger explicitly: `write(logger, () => ({message: 'ready'}))`.
Receiver methods support direct calls; `const write = logger.info` is rejected.

Events return void. Tracing handles event-write failures; they are not retroactively a typed Result. Explicit flush and consuming close return their own I/O errors. Flush is not fsync. Close releases this handle; children or installed access middleware retain the sink until their final owner is dropped. File creation appends and never silently truncates.

Ownership: logging borrows the logger and consumes the single-use factory/result. child consumes field data, flush borrows, close consumes its logger handle. Async request and started-task context uses per-poll tracing instrumentation; entered span guards never cross an await.
