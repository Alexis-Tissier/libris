#!/usr/bin/env bash
set -euo pipefail

OWNER="${LIBRIS_GITHUB_OWNER:-Alexis-Tissier}"
REPO="${LIBRIS_GITHUB_REPO:-libris}"
VERSION="$(node -p "require('./package.json').version")"
TAG="v${VERSION}"
DESCRIPTION="Bibliothèque personnelle locale pour livres physiques, avec suivi de lecture et recommandations."

cd "$(dirname "$0")/.."

# Refuse de publier si des données utilisateur sont présentes dans le projet.
if find . -type f \
  \( -iname '*.sqlite' -o -iname '*.sqlite3' -o -iname '*.db' -o -iname '*.db-wal' -o -iname '*.db-shm' -o -iname '.env' -o -iname '.env.*' \) \
  -not -path './.git/*' | grep -q .; then
  echo "ERREUR: base de données, fichier .env ou donnée locale détectée dans le projet."
  find . -type f \
    \( -iname '*.sqlite' -o -iname '*.sqlite3' -o -iname '*.db' -o -iname '*.db-wal' -o -iname '*.db-shm' -o -iname '.env' -o -iname '.env.*' \) \
    -not -path './.git/*'
  exit 1
fi

if ! command -v git >/dev/null 2>&1; then
  echo "Git est requis."
  exit 1
fi

if ! command -v gh >/dev/null 2>&1; then
  echo "GitHub CLI (gh) n'est pas installé."
  echo "Fedora: sudo dnf install -y gh"
  exit 1
fi

if ! gh auth status >/dev/null 2>&1; then
  echo "Connexion GitHub nécessaire. Une page de connexion va s'ouvrir."
  gh auth login -h github.com -p https -w
fi

if [ ! -d .git ]; then
  git init -b main
fi

git checkout -B main

git add .
if git diff --cached --quiet; then
  echo "Aucun changement à committer."
else
  git commit -m "release: Libris ${VERSION}"
fi

if gh repo view "${OWNER}/${REPO}" >/dev/null 2>&1; then
  echo "Dépôt ${OWNER}/${REPO} déjà présent."
  if ! git remote get-url origin >/dev/null 2>&1; then
    git remote add origin "https://github.com/${OWNER}/${REPO}.git"
  fi
else
  echo "Création du dépôt public ${OWNER}/${REPO}..."
  gh repo create "${OWNER}/${REPO}" \
    --public \
    --description "$DESCRIPTION" \
    --source . \
    --remote origin
fi

git push -u origin main

if git rev-parse "$TAG" >/dev/null 2>&1; then
  echo "Tag $TAG déjà présent localement."
else
  git tag -a "$TAG" -m "Libris ${VERSION}"
fi

git push origin "$TAG"

echo
echo "Publication lancée."
echo "GitHub Actions va compiler Windows, Linux et macOS puis créer la Release ${TAG}."
echo "https://github.com/${OWNER}/${REPO}/actions"
