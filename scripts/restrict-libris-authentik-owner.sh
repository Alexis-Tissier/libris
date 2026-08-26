#!/usr/bin/env bash
set -Eeuo pipefail

AUTH_DIR="${AUTH_DIR:-/opt/apps/authentik}"
STAMP="$(date +%Y%m%d-%H%M%S)"
BACKUP_DIR="${HOME}/libris-backups/authentik-access-${STAMP}"

mkdir -p "$BACKUP_DIR"
cd "$AUTH_DIR"

printf '\n======================================\n'
printf ' LIBRIS - ACCES AUTHENTIK PRIVE\n'
printf '======================================\n'

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
from authentik.core.models import Application, User
from authentik.policies.models import PolicyBinding

app = Application.objects.get(slug="libris")

active_users = list(User.objects.filter(is_active=True).order_by("pk"))
superusers = [user for user in active_users if getattr(user, "is_superuser", False)]

if len(superusers) != 1:
    print("ERREUR: impossible de déterminer automatiquement le compte propriétaire Libris.")
    print("Utilisateurs actifs Authentik:")
    for user in active_users:
        print(
            " -",
            f"pk={user.pk}",
            f"username={user.username!r}",
            f"name={getattr(user, 'name', '')!r}",
            f"superuser={getattr(user, 'is_superuser', False)}",
        )
    raise SystemExit(
        "Il faut exactement un superutilisateur actif pour appliquer la restriction automatiquement."
    )

owner = superusers[0]

# Une application sans binding est accessible à tous les utilisateurs authentifiés.
# Libris doit au contraire être privé : on remplace ses bindings par un binding
# direct vers le compte propriétaire uniquement.
deleted, _ = PolicyBinding.objects.filter(target=app).delete()
binding = PolicyBinding.objects.create(
    target=app,
    user=owner,
    order=0,
    enabled=True,
    negate=False,
)

bindings = list(PolicyBinding.objects.filter(target=app).select_related("user"))
if len(bindings) != 1 or bindings[0].user_id != owner.pk:
    raise SystemExit("La restriction Libris n'a pas été appliquée correctement")

print("application =", app.name, app.slug)
print("owner_pk =", owner.pk)
print("owner_username =", owner.username)
print("owner_name =", getattr(owner, "name", ""))
print("bindings_deleted =", deleted)
print("binding_pk =", binding.pk)
print("binding_user_only =", True)
print("LIBRIS_AUTHENTIK_OWNER_ONLY_OK")
PY

printf '\n======================================\n'
printf ' LIBRIS_AUTHENTIK_OWNER_ONLY_OK\n'
printf '======================================\n'
printf 'BACKUP=%s\n' "$BACKUP_DIR"
