<!-- tsrust-docs
{"id":"reference-standard-library-signals","group":"standard-library","order":20,"status":"partial","targets":["web","native"],"api":[{"receiver":"std:signals","name":"listen"},{"receiver":"std:signals","name":"receive"},{"receiver":"std:signals","name":"SignalListener"},{"receiver":"std:signals","name":"SignalError"}]}
-->
# Signals

Coordinate shutdown with SIGINT and SIGTERM. Register a listener before announcing readiness, then await receive. Cancelling a receive releases its loan without closing the listener. These signal registrations require a Unix target.
