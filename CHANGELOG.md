# Changelog

Toutes les évolutions publiques importantes de Libris sont documentées ici.

## 0.5.0 — 2026-08-07

Première version publique.

### Application

- bibliothèque locale de livres physiques ;
- recherche d'éditions via BnF, Google Books et Open Library ;
- gestion des statuts de lecture, progression, notes, avis, tags et prêts ;
- import/export JSON ;
- statistiques, profil littéraire et recommandations personnalisées ;
- regroupement logique des différentes éditions d'une même œuvre dans les recommandations sans fusionner les exemplaires de la bibliothèque ;
- priorité donnée aux éditions françaises dans la découverte et les recommandations.

### Publication

- suppression des données et chemins personnels du code public ;
- base SQLite utilisateur explicitement exclue du dépôt ;
- CI GitHub pour TypeScript et Rust ;
- release automatique Windows x64 (`.exe`, `.msi`) ;
- release automatique Linux x64 (`.AppImage`, `.deb`, `.rpm`) ;
- release automatique macOS Apple Silicon et Intel (`.dmg`).
