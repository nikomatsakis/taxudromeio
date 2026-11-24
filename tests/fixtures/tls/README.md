These certificates and the PKCS#12 private key are public test fixtures for
loopback protocol tests only. The bundle password is `fixture`. The server
certificate is valid for `localhost` from November 2026 to November 2027;
its lifetime is within macOS's limit for TLS leaf certificates.
The CA private key was discarded. Tests add the CA to an individual client;
they never install it in the macOS trust store or disable certificate checks.
Regenerate the fixtures when their validity window no longer covers test runs.

The public bundle uses SHA-1/3DES PKCS#12 protection for compatibility with both
macOS Security Framework and OpenSSL 3's default providers. It contains no private
credential. This packaging choice does not select TLS ciphers or disable server
certificate/hostname verification. Avoid OpenSSL's default `-legacy` export,
whose RC2 certificate encryption requires an extra provider on Linux.
