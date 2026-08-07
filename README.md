# Libris

Libris est une application de bureau locale pour gérer une **bibliothèque de livres physiques**, suivre ses lectures et obtenir des recommandations à partir de sa collection.

> Version publique actuelle : `0.5.0`.

## Fonctionnalités

- bibliothèque physique locale avec statuts **À lire**, **En cours**, **Lu** et **Abandonné** ;
- recherche par titre, auteur ou ISBN ;
- croisement de plusieurs catalogues bibliographiques, avec priorité aux éditions françaises ;
- gestion de l'édition physique : ISBN, éditeur, collection, date, pagination et couverture ;
- suivi de progression, notes, avis, tags, prêts et informations d'achat ;
- import et export JSON ;
- profil de lecture avec auteurs, genres et thèmes récurrents ;
- recommandations personnalisées qui excluent les œuvres déjà présentes, même sous une autre édition ou un autre ISBN ;
- fonctionnement sans compte avec une base SQLite locale.

## Télécharger

Les versions publiées dans **GitHub Releases** sont compilées automatiquement pour :

- **Windows x64** : `.exe` (NSIS) et `.msi` ;
- **Linux x64** : `.AppImage`, `.deb` et `.rpm` ;
- **macOS Apple Silicon** : `.dmg` ;
- **macOS Intel** : `.dmg`.

Les builds macOS utilisent une signature ad hoc mais ne sont pas notarisés avec un compte Apple Developer. Les builds Windows ne sont pas signés avec un certificat commercial : Windows SmartScreen peut donc afficher un avertissement lors de la première installation.

## Stack

- [Tauri 2](https://tauri.app/) pour l'application de bureau ;
- React + TypeScript + Vite pour l'interface ;
- Rust + rusqlite pour le moteur local et la persistance ;
- BnF, Google Books et Open Library comme sources de métadonnées bibliographiques.

## Confidentialité

La bibliothèque de l'utilisateur n'est **jamais stockée dans le dépôt GitHub**.

Libris place sa base SQLite dans le dossier de données local du système :

```text
Linux   : ~/.local/share/libris/libris.sqlite
Windows : %LOCALAPPDATA%\libris\libris.sqlite
macOS   : ~/Library/Application Support/libris/libris.sqlite
```

Le chemin exact dépend des conventions du système et peut être consulté depuis la page **Données** de Libris.

Aucun compte ni service d'analytics n'est nécessaire. Les catalogues externes sont interrogés uniquement pour les fonctions qui nécessitent des métadonnées. Voir [`docs/PRIVACY.md`](docs/PRIVACY.md).

## Développement

Prérequis : Node.js LTS, Rust/Cargo et les dépendances système de Tauri.

### Fedora

```bash
bash scripts/setup_fedora.sh
npm run tauri dev
```

Ou :

```bash
bash scripts/dev.sh
```

### Vérification

```bash
npm ci
npm run check
cd src-tauri
cargo check --locked
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
src-tauri/icons/      Icônes de l'application
scripts/              Développement, builds et publication
docs/                 Documentation
.github/workflows/    CI et releases multiplateformes
```

## Données et sauvegardes

Les bases SQLite, exports personnels, sauvegardes, fichiers `.env`, `node_modules`, `dist` et `src-tauri/target` sont exclus du dépôt par `.gitignore`.

## Licence

MIT — voir [`LICENSE`](LICENSE).
