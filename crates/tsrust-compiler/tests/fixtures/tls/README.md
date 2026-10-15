These public test credentials are deliberately checked in. Never use them outside local tests.

`generate.py` regenerates the CA, one server key, matching valid/expired/future/wrong-host certificates, and a mismatched key using OpenSSL. Positive tests trust only this CA. The server certificate covers localhost and 127.0.0.1; validity ends in 2120. Expired and future certificates exercise time verification without changing the system clock. The CA signing key is temporary and is not retained.
