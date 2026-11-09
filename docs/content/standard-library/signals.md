<!-- tsrust-docs
{"id":"reference-standard-library-signals","group":"standard-library","order":20,"status":"partial","targets":["web","native"],"api":[{"receiver":"std:signals","name":"listen"},{"receiver":"std:signals","name":"receive"},{"receiver":"std:signals","name":"SignalListener"},{"receiver":"std:signals","name":"SignalError"}]}
-->
# Signals

Coordinate shutdown with Unix SIGINT/SIGTERM or Windows console SIGINT/SIGBREAK. Register a listener before announcing readiness, then await receive. Cancelling a receive releases its loan without closing the listener. Registration requires an active async runtime on Unix or Windows. Other targets return unsupported.

Renamed imports preserve signal-operation contracts. Call `listen` and `receive` directly through their imported names; storing either operation itself as a function value is unsupported. Use an explicit async wrapper when a callable is needed.
