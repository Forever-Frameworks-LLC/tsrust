<!-- tsrust-docs
{"id":"reference-standard-library-path","group":"standard-library","order":20,"status":"partial","targets":["web","native"],"api":[{"receiver":"std:path","name":"join"},{"receiver":"std:path","name":"basename"},{"receiver":"std:path","name":"dirname"},{"receiver":"std:path","name":"extname"},{"receiver":"std:path","name":"isAbsolute"},{"receiver":"std:path","name":"normalize"},{"receiver":"std:path","name":"relative"},{"receiver":"std:path","name":"PathParts"},{"receiver":"std:path","name":"PathFormatOptions"},{"receiver":"std:path","name":"PathError"},{"receiver":"std:path","name":"parse"},{"receiver":"std:path","name":"format"},{"receiver":"std:path","name":"resolve"},{"receiver":"std:path","name":"separator"},{"receiver":"std:path","name":"delimiter"}]}
-->
# Path

Construct and inspect filesystem paths. Path operations manipulate names; they do not prove a file exists or grant access to it. Use filesystem operations for those checks.
