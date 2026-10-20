<!-- tsrust-docs
{"id":"reference-standard-library-numeric","group":"standard-library","order":20,"status":"partial","targets":["web","native"],"api":[{"receiver":"std:numeric","name":"toInt"},{"receiver":"std:numeric","name":"toNumber"},{"receiver":"std:numeric","name":"toFloat32"},{"receiver":"std:numeric","name":"checkSigned"},{"receiver":"std:numeric","name":"checkUnsigned"},{"receiver":"std:numeric","name":"parseInt"},{"receiver":"std:numeric","name":"NumericError"}]}
-->
# Numeric

Convert numbers at explicit boundaries. std:numeric.parseInt validates a complete signed decimal string and reports syntax or range failures. The global parseInt has different prefix-parsing behavior.
