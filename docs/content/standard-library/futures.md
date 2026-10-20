<!-- tsrust-docs
{"id":"reference-standard-library-futures","group":"standard-library","order":20,"status":"partial","targets":["web","native"],"api":[{"receiver":"std:futures","name":"delay"},{"receiver":"std:futures","name":"yield"},{"receiver":"std:futures","name":"race"},{"receiver":"std:futures","name":"allSettled"},{"receiver":"std:futures","name":"Job"},{"receiver":"std:futures","name":"AsyncError"},{"receiver":"std:futures","name":"Channel"},{"receiver":"std:futures","name":"Sender"},{"receiver":"TaskMethods","name":"start"},{"receiver":"TaskMethods","name":"timeout"},{"receiver":"Job","name":"cancel"},{"receiver":"Job","name":"timeout"},{"receiver":"AsyncChannel","name":"sender"},{"receiver":"AsyncChannel","name":"send"},{"receiver":"AsyncChannel","name":"receive"},{"receiver":"AsyncChannel","name":"close"},{"receiver":"AsyncSender","name":"send"}]}
-->
# Tasks and async queues

Tasks and async channels preserve ownership, cancellation and cleanup. Use the documented send/receive and join contracts; do not assume resources can cross task boundaries.
