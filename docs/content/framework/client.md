<!-- tsrust-docs
{"id":"reference-framework-client","group":"framework","order":20,"status":"partial","targets":["web","native"],"api":[{"receiver":"HttpClient","name":"send"},{"receiver":"HttpBody","name":"next"},{"receiver":"HttpBody","name":"text"}]}
-->
# HTTP client and bodies

Use the client and body APIs with their explicit ownership and cancellation rules. Streaming support depends on the concrete operation and target; it is not implied by a body type.
