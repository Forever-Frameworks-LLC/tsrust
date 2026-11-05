<!-- tsrust-docs
{"id":"reference-standard-library-env","group":"standard-library","order":20,"status":"partial","targets":["web","native"],"api":[{"receiver":"std:env","name":"EnvError"},{"receiver":"std:env","name":"get"},{"receiver":"std:env","name":"args"},{"receiver":"std:env","name":"EnvVar"},{"receiver":"std:env","name":"vars"},{"receiver":"std:env","name":"currentDir"},{"receiver":"std:env","name":"currentExe"},{"receiver":"std:env","name":"tempDir"},{"receiver":"std:env","name":"homeDir"}]}
-->
# Env

Read process environment values through std:env. Treat absent values separately from empty strings, then validate configuration before constructing application services.
