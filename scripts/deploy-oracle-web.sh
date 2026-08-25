#!/usr/bin/env bash
set -Eeuo pipefail

APP_DIR="${APP_DIR:-/opt/apps/libris}"
DATA_DIR="${DATA_DIR:-/opt/apps/libris-data}"
AUTH_DIR="${AUTH_DIR:-/opt/apps/authentik}"
CADDYFILE="${CADDYFILE:-/etc/caddy/Caddyfile}"
DOMAIN="${DOMAIN:-libris.alexis-tissier.fr}"
PORT="${PORT:-8030}"
STAMP="$(date +%Y%m%d-%H%M%S)"
BACKUP_DIR="${HOME}/libris-backups/deploy-${STAMP}"

cd "$APP_DIR"
mkdir -p "$BACKUP_DIR"
sudo mkdir -p "$DATA_DIR/libris"

printf '\n======================================\n'
printf ' LIBRIS - DEPLOIEMENT ORACLE WEB\n'
printf '======================================\n'

printf '\n=== 1. SOURCE ===\n'
git fetch origin main
git merge --ff-only origin/main
printf 'HEAD=%s\n' "$(git rev-parse --short HEAD)"

printf '\n=== 2. BACKUP DONNEES + CONFIG ===\n'
if sudo test -f "$DATA_DIR/libris/libris.sqlite"; then
  sudo cp -a "$DATA_DIR/libris/libris.sqlite" "$BACKUP_DIR/libris.sqlite"
  sudo test -f "$DATA_DIR/libris/libris.sqlite-wal" && sudo cp -a "$DATA_DIR/libris/libris.sqlite-wal" "$BACKUP_DIR/libris.sqlite-wal" || true
  sudo test -f "$DATA_DIR/libris/libris.sqlite-shm" && sudo cp -a "$DATA_DIR/libris/libris.sqlite-shm" "$BACKUP_DIR/libris.sqlite-shm" || true
fi
sudo cp -a "$CADDYFILE" "$BACKUP_DIR/Caddyfile.before-libris"
printf 'BACKUP=%s\n' "$BACKUP_DIR"

printf '\n=== 3. BUILD WEB ARM64 ===\n'
sudo docker compose -f compose.oracle.yml build

printf '\n=== 4. DEMARRAGE ===\n'
sudo docker compose -f compose.oracle.yml up -d

for _ in $(seq 1 40); do
  if curl -fsS "http://127.0.0.1:${PORT}/api/health" >/dev/null 2>&1; then
    break
  fi
  sleep 2
done
curl -fsS "http://127.0.0.1:${PORT}/api/health"
echo

printf '\n=== 5. AUTHENTIK ===\n'
cd "$AUTH_DIR"
if sudo docker compose ps --services | grep -qx postgresql; then
  sudo docker compose exec -T postgresql sh -c '
    PGUSER_VALUE="${POSTGRES_USER:-authentik}"
    PGDB_VALUE="${POSTGRES_DB:-authentik}"
    exec pg_dump -U "$PGUSER_VALUE" "$PGDB_VALUE"
  ' </dev/null | gzip -c > "$BACKUP_DIR/authentik.sql.gz"
  test -s "$BACKUP_DIR/authentik.sql.gz"
  echo "AUTHENTIK_DB_BACKUP_OK"
fi

sudo docker compose exec -T server ak shell <<'PY'
from authentik.core.models import Application
from authentik.outposts.models import Outpost
from authentik.providers.proxy.models import ProxyMode, ProxyProvider

EXTERNAL = "https://libris.alexis-tissier.fr"
template = ProxyProvider.objects.get(name="Horizon Proxy")

provider, provider_created = ProxyProvider.objects.get_or_create(
    name="Libris Proxy",
    defaults={
        "authentication_flow": template.authentication_flow,
        "authorization_flow": template.authorization_flow,
        "invalidation_flow": template.invalidation_flow,
        "mode": ProxyMode.FORWARD_SINGLE,
        "external_host": EXTERNAL,
        "internal_host": "",
    },
)
provider.authentication_flow = template.authentication_flow
provider.authorization_flow = template.authorization_flow
provider.invalidation_flow = template.invalidation_flow
provider.mode = ProxyMode.FORWARD_SINGLE
provider.external_host = EXTERNAL
provider.internal_host = ""
provider.save()
provider.set_oauth_defaults()
provider.save()

app, app_created = Application.objects.get_or_create(
    slug="libris",
    defaults={"name": "Libris", "provider": provider, "meta_launch_url": EXTERNAL},
)
app.name = "Libris"
app.provider = provider
app.meta_launch_url = EXTERNAL
app.save()

outpost = Outpost.objects.get(name="authentik Embedded Outpost")
outpost.providers.add(provider)

print("provider_created =", provider_created)
print("application_created =", app_created)
print("provider =", provider.name, provider.mode, provider.external_host)
print("application =", app.name, app.slug)
print("outpost_has_libris =", outpost.providers.filter(pk=provider.pk).exists())
PY

printf '\n=== 6. CADDY ===\n'
sudo python3 - "$CADDYFILE" "$DOMAIN" "$PORT" <<'PY'
from pathlib import Path
import sys

path = Path(sys.argv[1])
domain = sys.argv[2]
port = sys.argv[3]
text = path.read_text(encoding="utf-8")

new_block = f'''{domain} {{
\troute {{
\t\treverse_proxy /outpost.goauthentik.io/* 127.0.0.1:9000

\t\tforward_auth 127.0.0.1:9000 {{
\t\t\turi /outpost.goauthentik.io/auth/caddy
\t\t\tcopy_headers X-Authentik-Username X-Authentik-Groups X-Authentik-Entitlements X-Authentik-Email X-Authentik-Name X-Authentik-Uid X-Authentik-Jwt X-Authentik-Meta-Jwks X-Authentik-Meta-Outpost X-Authentik-Meta-Provider X-Authentik-Meta-App X-Authentik-Meta-Version
\t\t}}

\t\treverse_proxy 127.0.0.1:{port}
\t}}
}}'''

marker = domain + " {"
start = text.find(marker)
if start < 0:
    if text and not text.endswith("\n"):
        text += "\n"
    text += "\n" + new_block + "\n"
else:
    brace = text.find("{", start)
    depth = 0
    end = None
    for i in range(brace, len(text)):
        if text[i] == "{":
            depth += 1
        elif text[i] == "}":
            depth -= 1
            if depth == 0:
                end = i + 1
                break
    if end is None:
        raise SystemExit(f"Bloc Caddy incomplet pour {domain}")
    text = text[:start] + new_block + text[end:]

path.write_text(text, encoding="utf-8")
print("CADDY_LIBRIS_WRITTEN")
PY

sudo caddy fmt --overwrite "$CADDYFILE"
sudo caddy validate --config "$CADDYFILE"
sudo systemctl reload caddy

printf '\n=== 7. VERIFICATIONS ===\n'
printf 'Caddy: %s\n' "$(systemctl is-active caddy)"
printf 'Libris direct: '
curl -fsS "http://127.0.0.1:${PORT}/api/health"
echo

printf '\nPublic sans cookie (attendu: 302 vers Authentik):\n'
curl -skD - -o /dev/null "https://${DOMAIN}/" | grep -Ei 'HTTP/|location:|server:' | head -12

printf '\nOutpost (attendu: 204):\n'
curl -skD - -o /dev/null "https://${DOMAIN}/outpost.goauthentik.io/ping" | grep -Ei 'HTTP/|server:' | head -8

printf '\nConteneur:\n'
cd "$APP_DIR"
sudo docker compose -f compose.oracle.yml ps

printf '\n======================================\n'
printf ' LIBRIS_DEPLOY_OK\n'
printf '======================================\n'
printf 'BACKUP=%s\n' "$BACKUP_DIR"
printf 'DATA=%s\n' "$DATA_DIR/libris/libris.sqlite"
