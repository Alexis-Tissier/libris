# Confidentialité

Libris est conçu comme une application locale.

## Données de bibliothèque

Les livres, notes, avis, statuts, informations d'achat, prêts et autres données personnelles sont enregistrés dans une base SQLite située dans le dossier de données local de l'utilisateur, sous `libris/libris.sqlite`.

Emplacements habituels :

- Linux : `~/.local/share/libris/libris.sqlite` ;
- Windows : `%LOCALAPPDATA%\libris\libris.sqlite` ;
- macOS : `~/Library/Application Support/libris/libris.sqlite`.

Ces données ne font pas partie du dépôt Git et les formats SQLite ainsi que leurs fichiers annexes sont explicitement ignorés par `.gitignore`.

## Services externes

Libris interroge des catalogues bibliographiques externes lorsque l'utilisateur recherche un livre, récupère une couverture ou demande des recommandations nécessitant des métadonnées. Les sources actuellement utilisées comprennent la BnF, Google Books et Open Library.

Libris n'intègre pas de compte utilisateur, de télémétrie ou de service d'analytics.

## Import et export

Les exports JSON sont créés uniquement à la demande de l'utilisateur, à l'emplacement qu'il choisit. Ils peuvent contenir les informations de sa bibliothèque et doivent donc être traités comme des données personnelles.
