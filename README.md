# Libris

Libris est une application pour gérer une **bibliothèque de livres physiques**, suivre ses lectures et obtenir des recommandations à partir de sa collection.

> Version publique actuelle : `0.5.0`.

Libris existe désormais dans deux modes qui partagent la même interface et le même moteur métier :

- **bureau** : Tauri + SQLite locale ;
- **web privé** : React + serveur Rust + SQLite persistante, prévu pour `libris.alexis-tissier.fr` derrière Authentik.

## Fonctionnalités

- bibliothèque physique avec statuts **À lire**, **En cours**, **Lu** et **Abandonné** ;
- recherche par titre, auteur ou ISBN ;
- croisement de plusieurs catalogues bibliographiques, avec priorité aux éditions françaises ;
- gestion de l'édition physique : ISBN, éditeur, collection, date, pagination et couverture ;
- suivi de progression, notes, avis, tags, prêts et informations d'achat ;
- import et export JSON ;
- profil de lecture avec auteurs, genres et thèmes récurrents ;
- recommandations personnalisées qui excluent les œuvres déjà présentes, même sous une autre édition ou un autre ISBN.

## Application de bureau

Les versions publiées dans **GitHub Releases** sont compilées automatiquement pour :

- **Windows x64** : `.exe` (NSIS) et `.msi` ;
- **Linux x64** : `.AppImage`, `.deb` et `.rpm` ;
- **macOS Apple Silicon** : `.dmg` ;
- **macOS Intel** : `.dmg`.

Les builds macOS utilisent une signature ad hoc mais ne sont pas notarisés avec un compte Apple Developer. Les builds Windows ne sont pas signés avec un certificat commercial : Windows SmartScreen peut donc afficher un avertissement lors de la première installation.

## Mode web privé / Oracle

Le mode web conserve le moteur Rust de Libris mais remplace les commandes Tauri par une API HTTP locale au conteneur. Le frontend choisit automatiquement le transport Tauri ou web au démarrage.

Déploiement prévu :

```text
libris.alexis-tissier.fr
        ↓
      Caddy
        ↓
 Authentik forward-auth
        ↓
127.0.0.1:8030
        ↓
 libris-server + React
        ↓
/opt/apps/libris-data/libris/libris.sqlite
```

Le conteneur Oracle est défini dans [`compose.oracle.yml`](compose.oracle.yml) et construit avec [`Dockerfile.oracle`](Dockerfile.oracle).

Sur Oracle, depuis `/opt/apps/libris` :

```bash
bash scripts/deploy-oracle-web.sh
```

Le script est idempotent : il construit l'image ARM64, conserve la base dans `/opt/apps/libris-data`, crée/met à jour l'application **Libris** dans Authentik, configure le forward-auth Caddy et vérifie le healthcheck.

### Migrer la bibliothèque du PC vers Oracle

Depuis Linux/Fedora, après le premier déploiement web :

```bash
bash scripts/migrate-local-library-to-oracle.sh
```

Le script prend un snapshot SQLite cohérent avec l'API `backup`, sauvegarde la base déjà présente sur Oracle, remplace la base à froid puis vérifie son intégrité et le nombre de livres.

Par défaut, la source est :

```text
~/.local/share/libris/libris.sqlite
```

## Mobile

Le web est responsive : sous 760 px, la barre latérale devient un header compact avec menu burger, les grilles passent en une ou deux colonnes, le drawer d'un livre devient plein écran et les contrôles de recherche/recommandation sont adaptés au tactile.

## Stack

- [Tauri 2](https://tauri.app/) pour l'application de bureau ;
- React + TypeScript + Vite pour l'interface ;
- Rust + rusqlite pour le moteur et la persistance ;
- Axum pour le serveur web privé ;
- BnF, Google Books et Open Library comme sources de métadonnées bibliographiques ;
- Docker + Caddy + Authentik pour le déploiement Oracle.

## Confidentialité

La bibliothèque de l'utilisateur n'est **jamais stockée dans le dépôt GitHub**.

En mode bureau, Libris place sa base SQLite dans le dossier de données local du système :

```text
Linux   : ~/.local/share/libris/libris.sqlite
Windows : %LOCALAPPDATA%\libris\libris.sqlite
macOS   : ~/Library/Application Support/libris/libris.sqlite
```

En mode Oracle, le volume persistant est monté depuis :

```text
/opt/apps/libris-data
```

et la base utilisée dans le conteneur est :

```text
/data/libris/libris.sqlite
```

Le site public est prévu pour être protégé par Authentik avant d'atteindre le serveur Libris.

## Développement

Prérequis bureau : Node.js LTS, Rust/Cargo et les dépendances système de Tauri.

### Fedora / bureau

```bash
bash scripts/setup_fedora.sh
npm run tauri dev
```

Ou :

```bash
bash scripts/dev.sh
```

### Frontend web

```bash
npm ci
npm run dev
```

### Vérification

```bash
npm ci
npm run check
npm run build
cd src-tauri && cargo check --locked
cd ../server && cargo check
```

## Compiler sous Linux

```bash
bash scripts/build_linux.sh
```

Les paquets sont générés dans `src-tauri/target/release/bundle/`.

La compilation et l'installation d'une nouvelle version ne suppriment pas la base SQLite existante.

## Releases multiplateformes

Le workflow [`.github/workflows/release.yml`](.github/workflows/release.yml) compile automatiquement Libris sur les runners natifs GitHub pour Windows, Linux et les deux architectures macOS.

Une publication est déclenchée par un tag `v*`, par exemple :

```bash
git tag -a v0.5.0 -m "Libris 0.5.0"
git push origin v0.5.0
```

## Structure

```text
src/                  Interface React/TypeScript
src-tauri/src/        Moteur Rust, SQLite et catalogues
server/               Serveur HTTP Rust pour le mode web
scripts/              Développement, builds, migration et déploiement
docs/                 Documentation
.github/workflows/    CI et releases multiplateformes
```

## Données et sauvegardes

Les bases SQLite, exports personnels, sauvegardes, fichiers `.env`, `node_modules`, `dist`, `server/target` et `src-tauri/target` sont exclus du contexte de build ou du dépôt.

## Licence

MIT — voir [`LICENSE`](LICENSE).
