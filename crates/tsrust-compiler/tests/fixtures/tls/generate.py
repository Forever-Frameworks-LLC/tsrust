"""Regenerate public, test-only credentials. Requires OpenSSL; never use these keys in production."""
from pathlib import Path
import subprocess
import tempfile

output = Path(__file__).resolve().parent

def openssl(*arguments):
    subprocess.run(['openssl', *map(str, arguments)], check=True, capture_output=True)

with tempfile.TemporaryDirectory() as scratch:
    root = Path(scratch)
    openssl('req', '-x509', '-newkey', 'rsa:2048', '-nodes', '-keyout', root/'ca.key',
            '-out', output/'ca.pem', '-subj', '/CN=Tsrust test CA', '-days', '36500',
            '-addext', 'basicConstraints=critical,CA:TRUE', '-addext', 'keyUsage=critical,keyCertSign,cRLSign')
    (root/'index').write_text('')
    (root/'serial').write_text('01\n')
    openssl('genpkey', '-algorithm', 'RSA', '-pkeyopt', 'rsa_keygen_bits:2048', '-out', output/'server.key')
    for name, san, start, end in [
        ('server', 'DNS:localhost,IP:127.0.0.1', '20200101000000Z', '21200101000000Z'),
        ('wrong-host', 'DNS:other.invalid', '20200101000000Z', '21200101000000Z'),
        ('expired', 'DNS:localhost,IP:127.0.0.1', '20200101000000Z', '20210101000000Z'),
        ('future', 'DNS:localhost,IP:127.0.0.1', '21200101000000Z', '21210101000000Z'),
    ]:
        config = root/'ca.conf'
        config.write_text(f'''[ca]
default_ca=local
[local]
database={root/'index'}
serial={root/'serial'}
new_certs_dir={root}
certificate={output/'ca.pem'}
private_key={root/'ca.key'}
default_md=sha256
policy=policy
unique_subject=no
x509_extensions=extensions
[policy]
commonName=supplied
[extensions]
basicConstraints=critical,CA:FALSE
keyUsage=critical,digitalSignature,keyEncipherment
extendedKeyUsage=serverAuth
subjectAltName={san}
''')
        openssl('req', '-new', '-key', output/'server.key', '-out', root/'server.csr', '-subj', '/CN=localhost')
        openssl('ca', '-batch', '-notext', '-config', config, '-in', root/'server.csr', '-out', output/f'{name}.pem', '-startdate', start, '-enddate', end)
    openssl('genpkey', '-algorithm', 'RSA', '-pkeyopt', 'rsa_keygen_bits:2048', '-out', output/'mismatch.key')
