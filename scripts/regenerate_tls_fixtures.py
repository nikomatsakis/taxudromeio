"""Regenerate public localhost-only TLS test fixtures using OpenSSL 3."""

from pathlib import Path
import subprocess
import tempfile


def main():
    destination = Path(__file__).resolve().parent.parent / "tests/fixtures/tls"
    with tempfile.TemporaryDirectory(prefix="taxudromeio-test-tls-") as temporary:
        directory = Path(temporary)
        (directory / "extensions").write_text(
            "subjectAltName=DNS:localhost\n"
            "basicConstraints=critical,CA:FALSE\n"
            "keyUsage=critical,digitalSignature,keyEncipherment\n"
            "extendedKeyUsage=serverAuth\n"
        )
        commands = [
            [
                "openssl", "req", "-x509", "-newkey", "rsa:2048", "-nodes",
                "-keyout", "ca.key", "-out", "ca.pem", "-days", "3650",
                "-subj", "/CN=Taxudromeio test CA",
                "-addext", "basicConstraints=critical,CA:TRUE",
                "-addext", "keyUsage=critical,keyCertSign,cRLSign",
            ],
            [
                "openssl", "req", "-new", "-newkey", "rsa:2048", "-nodes",
                "-keyout", "server.key", "-out", "server.csr",
                "-subj", "/CN=localhost",
            ],
            [
                "openssl", "x509", "-req", "-in", "server.csr",
                "-CA", "ca.pem", "-CAkey", "ca.key", "-CAcreateserial",
                "-out", "server.pem", "-days", "365", "-extfile", "extensions",
            ],
            [
                "openssl", "pkcs12", "-export", "-inkey", "server.key",
                "-in", "server.pem", "-certfile", "ca.pem", "-out", "server.p12",
                "-keypbe", "PBE-SHA1-3DES", "-certpbe", "PBE-SHA1-3DES",
                "-macalg", "sha1",
                "-passout", "pass:fixture",
            ],
        ]
        for command in commands:
            subprocess.run(command, cwd=directory, check=True, capture_output=True)
        for name in ("ca.pem", "server.p12"):
            (destination / name).write_bytes((directory / name).read_bytes())
    print("Regenerated public TLS fixtures; leaf certificate expires in 365 days.")


if __name__ == "__main__":
    main()
