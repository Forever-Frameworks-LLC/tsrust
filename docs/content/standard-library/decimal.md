<!-- tsrust-docs
{"id":"reference-standard-library-decimal","group":"standard-library","order":20,"status":"partial","targets":["web","native"],"api":[{"receiver":"std:decimal","name":"parseDecimal"},{"receiver":"std:decimal","name":"fromInt"},{"receiver":"std:decimal","name":"format"},{"receiver":"std:decimal","name":"formatFixed"},{"receiver":"std:decimal","name":"add"},{"receiver":"std:decimal","name":"subtract"},{"receiver":"std:decimal","name":"multiply"},{"receiver":"std:decimal","name":"divideExact"},{"receiver":"std:decimal","name":"compare"},{"receiver":"std:decimal","name":"toInt"},{"receiver":"std:decimal","name":"toNumber"},{"receiver":"std:decimal","name":"Decimal"},{"receiver":"std:decimal","name":"DecimalError"}]}
-->
# Decimal

Keep base-10 values exact with Decimal. parseDecimal validates the whole string; arithmetic returns checked results. toNumber rejects values that binary64 cannot represent exactly, including 0.1. Serialize exact values as decimal text.
