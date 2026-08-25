#!/usr/bin/env bash
set -Eeuo pipefail

ORACLE_HOST="${ORACLE_HOST:-oracle-server}"
LOCAL_DB="${LOCAL_DB:-$HOME/.local/share/libris/libris.sqlite}"
REMOTE_APP="${REMOTE_APP:-/opt/apps/libris}"
REMOTE_DATA="${REMOTE_DATA:-/opt/apps/libris-data/libris/libris.sqlite}"
STAMP="$(date +%Y%m%d-%H%M%S)"
TMP_DIR="$(mktemp -d)"
BACKUP_DB="$TMP_DIR/libris.sqlite"
trap 'rm -rf "$TMP_DIR"' EXIT

printf '\n======================================\n'
printf ' LIBRIS - MIGRATION DONNEES -> ORACLE\n'
printf '======================================\n'
printf 'Source : %s\n' "$LOCAL_DB"
printf 'Oracle : %s\n' "$ORACLE_HOST"

if [ ! -f "$LOCAL_DB" ]; then
  echo "ERREUR: base Libris locale introuvable: $LOCAL_DB" >&2
  exit 1
fi

printf '\n=== 1. SNAPSHOT SQLITE LOCAL ===\n'
python3 - "$LOCAL_DB" "$BACKUP_DB" <<'PY'
import sqlite3
import sys

source, target = sys.argv[1], sys.argv[2]
src = sqlite3.connect(f"file:{source}?mode=ro", uri=True)
dst = sqlite3.connect(target)
with dst:
    src.backup(dst)
check = dst.execute("PRAGMA integrity_check").fetchone()[0]
books = dst.execute("SELECT COUNT(*) FROM books").fetchone()[0]
print(f"integrity={check}")
print(f"books={books}")
if check != "ok":
    raise SystemExit("Snapshot SQLite invalide")
src.close()
dst.close()
PY

printf '\n=== 2. COPIE VERS ORACLE ===\n'
scp "$BACKUP_DB" "$ORACLE_HOST:/tmp/libris-migration-${STAMP}.sqlite"

printf '\n=== 3. INSTALLATION ATOMIQUE ===\n'
ssh "$ORACLE_HOST" "STAMP='$STAMP' REMOTE_APP='$REMOTE_APP' REMOTE_DATA='$REMOTE_DATA' bash -s" <<'REMOTE'
set -Eeuo pipefail

SOURCE="/tmp/libris-migration-${STAMP}.sqlite"
BACKUP_DIR="$HOME/libris-backups/data-${STAMP}"
mkdir -p "$BACKUP_DIR"

if sudo test -f "$REMOTE_DATA"; then
  sudo cp -a "$REMOTE_DATA" "$BACKUP_DIR/libris.sqlite.before-migration"
  sudo test -f "${REMOTE_DATA}-wal" && sudo cp -a "${REMOTE_DATA}-wal" "$BACKUP_DIR/libris.sqlite-wal.before-migration" || true
  sudo test -f "${REMOTE_DATA}-shm" && sudo cp -a "${REMOTE_DATA}-shm" "$BACKUP_DIR/libris.sqlite-shm.before-migration" || true
fi

cd "$REMOTE_APP"
sudo docker compose -f compose.oracle.yml stop libris
sudo mkdir -p "$(dirname "$REMOTE_DATA")"
sudo install -m 0644 "$SOURCE" "$REMOTE_DATA"
sudo rm -f "${REMOTE_DATA}-wal" "${REMOTE_DATA}-shm"
rm -f "$SOURCE"

sudo docker compose -f compose.oracle.yml up -d
for _ in $(seq 1 30); do
  if curl -fsS http://127.0.0.1:8030/api/health >/dev/null 2>&1; then
    break
  fi
  sleep 2
done
curl -fsS http://127.0.0.1:8030/api/health
echo

python3 - "$REMOTE_DATA" <<'PY'
import sqlite3
import sys
path = sys.argv[1]
conn = sqlite3.connect(f"file:{path}?mode=ro", uri=True)
print("integrity=", conn.execute("PRAGMA integrity_check").fetchone()[0])
print("books=", conn.execute("SELECT COUNT(*) FROM books").fetchone()[0])
print("reading=", conn.execute("SELECT COUNT(*) FROM books WHERE status='reading'").fetchone()[0])
print("read=", conn.execute("SELECT COUNT(*) FROM books WHERE status='read'").fetchone()[0])
conn.close()
PY

echo "REMOTE_BACKUP=$BACKUP_DIR"
REMOTE

printf '\n======================================\n'
printf ' LIBRIS_DATA_MIGRATION_OK\n'
printf '======================================\n'
