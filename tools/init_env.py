"""Create private local settings once; run through compose.setup.yml."""
import base64
from pathlib import Path
import secrets

root = Path('/project')
target = root / '.env'
if target.exists():
    raise SystemExit('.env already exists; existing settings were preserved')
text = (root / '.env.example').read_text(encoding='utf-8')
text = text.replace('POSTGRES_PASSWORD=\n', 'POSTGRES_PASSWORD='+secrets.token_hex(24)+'\n')
text = text.replace('APP_KEY=\n', 'APP_KEY=base64:'+base64.b64encode(secrets.token_bytes(32)).decode()+'\n')
with target.open('x', encoding='utf-8', newline='\n') as stream:
    stream.write(text)
target.chmod(0o600)
print('Created private .env; secret values are not printed')
