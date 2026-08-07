use crate::models::{Book, DashboardStats, NamedCount};
use rusqlite::{params, params_from_iter, Connection, Row};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

pub fn data_dir() -> Result<PathBuf, String> {
    let base = dirs::data_local_dir().ok_or_else(|| "Impossible de trouver le dossier de données local".to_string())?;
    let dir = base.join("libris");
    fs::create_dir_all(&dir).map_err(|e| format!("Impossible de créer le dossier Libris : {e}"))?;
    Ok(dir)
}

pub fn database_path() -> Result<PathBuf, String> {
    Ok(data_dir()?.join("libris.sqlite"))
}

fn connect() -> Result<Connection, String> {
    let path = database_path()?;
    let conn = Connection::open(path).map_err(|e| format!("Impossible d’ouvrir la base Libris : {e}"))?;
    conn.pragma_update(None, "foreign_keys", "ON").map_err(|e| e.to_string())?;
    conn.pragma_update(None, "journal_mode", "WAL").map_err(|e| e.to_string())?;
    conn.busy_timeout(std::time::Duration::from_secs(5)).map_err(|e| e.to_string())?;
    Ok(conn)
}

pub fn init() -> Result<(), String> {
    let conn = connect()?;
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS books (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            work_key TEXT,
            edition_key TEXT,
            source TEXT NOT NULL DEFAULT '',
            source_id TEXT NOT NULL DEFAULT '',
            isbn TEXT,
            title TEXT NOT NULL,
            authors TEXT NOT NULL DEFAULT '',
            publisher TEXT NOT NULL DEFAULT '',
            collection TEXT NOT NULL DEFAULT '',
            published_date TEXT NOT NULL DEFAULT '',
            format TEXT NOT NULL DEFAULT '',
            description TEXT NOT NULL DEFAULT '',
            subjects TEXT NOT NULL DEFAULT '',
            publish_year INTEGER,
            pages INTEGER,
            language TEXT NOT NULL DEFAULT 'fr',
            cover_url TEXT NOT NULL DEFAULT '',
            cover_path TEXT NOT NULL DEFAULT '',
            status TEXT NOT NULL DEFAULT 'wishlist',
            owned INTEGER NOT NULL DEFAULT 0,
            rating REAL,
            review TEXT NOT NULL DEFAULT '',
            liked TEXT NOT NULL DEFAULT '',
            disliked TEXT NOT NULL DEFAULT '',
            location TEXT NOT NULL DEFAULT '',
            purchase_price REAL,
            purchase_date TEXT NOT NULL DEFAULT '',
            started_at TEXT NOT NULL DEFAULT '',
            finished_at TEXT NOT NULL DEFAULT '',
            progress_pages INTEGER NOT NULL DEFAULT 0,
            loaned_to TEXT NOT NULL DEFAULT '',
            tags TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        );
        CREATE UNIQUE INDEX IF NOT EXISTS idx_books_isbn
            ON books(isbn) WHERE isbn IS NOT NULL AND isbn != '';
        CREATE INDEX IF NOT EXISTS idx_books_status ON books(status);
        CREATE INDEX IF NOT EXISTS idx_books_title ON books(title);

        CREATE TABLE IF NOT EXISTS recommendation_feedback (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            candidate_key TEXT NOT NULL,
            action TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        );

        CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );
        "#,
    )
    .map_err(|e| format!("Impossible d’initialiser la base Libris : {e}"))?;

    ensure_column(&conn, "books", "source", "TEXT NOT NULL DEFAULT ''")?;
    ensure_column(&conn, "books", "source_id", "TEXT NOT NULL DEFAULT ''")?;
    ensure_column(&conn, "books", "publisher", "TEXT NOT NULL DEFAULT ''")?;
    ensure_column(&conn, "books", "collection", "TEXT NOT NULL DEFAULT ''")?;
    ensure_column(&conn, "books", "published_date", "TEXT NOT NULL DEFAULT ''")?;
    ensure_column(&conn, "books", "format", "TEXT NOT NULL DEFAULT ''")?;
    Ok(())
}


fn ensure_column(conn: &Connection, table: &str, column: &str, definition: &str) -> Result<(), String> {
    let mut statement = conn
        .prepare(&format!("PRAGMA table_info({table})"))
        .map_err(|error| error.to_string())?;
    let columns = statement
        .query_map([], |row| row.get::<_, String>(1))
        .map_err(|error| error.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;
    if !columns.iter().any(|existing| existing == column) {
        conn.execute(
            &format!("ALTER TABLE {table} ADD COLUMN {column} {definition}"),
            [],
        )
        .map_err(|error| format!("Migration de la base Libris impossible ({column}) : {error}"))?;
    }
    Ok(())
}

fn row_to_book(row: &Row<'_>) -> rusqlite::Result<Book> {
    Ok(Book {
        id: row.get("id")?,
        work_key: row.get("work_key")?,
        edition_key: row.get("edition_key")?,
        source: row.get("source")?,
        source_id: row.get("source_id")?,
        isbn: row.get("isbn")?,
        title: row.get("title")?,
        authors: row.get("authors")?,
        publisher: row.get("publisher")?,
        collection: row.get("collection")?,
        published_date: row.get("published_date")?,
        format: row.get("format")?,
        description: row.get("description")?,
        subjects: row.get("subjects")?,
        publish_year: row.get("publish_year")?,
        pages: row.get("pages")?,
        language: row.get("language")?,
        cover_url: row.get("cover_url")?,
        cover_path: row.get("cover_path")?,
        status: row.get("status")?,
        owned: row.get::<_, i64>("owned")? != 0,
        rating: row.get("rating")?,
        review: row.get("review")?,
        liked: row.get("liked")?,
        disliked: row.get("disliked")?,
        location: row.get("location")?,
        purchase_price: row.get("purchase_price")?,
        purchase_date: row.get("purchase_date")?,
        started_at: row.get("started_at")?,
        finished_at: row.get("finished_at")?,
        progress_pages: row.get("progress_pages")?,
        loaned_to: row.get("loaned_to")?,
        tags: row.get("tags")?,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
    })
}

pub fn list_books(filter: Option<&str>, query: Option<&str>) -> Result<Vec<Book>, String> {
    init()?;
    let conn = connect()?;
    let mut sql = String::from("SELECT * FROM books WHERE 1=1");
    let mut values: Vec<String> = Vec::new();

    if let Some(filter) = filter.filter(|f| !f.is_empty() && *f != "all") {
        if filter == "owned" {
            sql.push_str(" AND owned=1");
        } else {
            sql.push_str(" AND status=?");
            values.push(filter.to_string());
        }
    }

    if let Some(query) = query.map(str::trim).filter(|q| !q.is_empty()) {
        sql.push_str(" AND (title LIKE ? OR authors LIKE ? OR publisher LIKE ? OR collection LIKE ? OR isbn LIKE ? OR subjects LIKE ? OR tags LIKE ?)");
        let token = format!("%{query}%");
        values.extend([
            token.clone(), token.clone(), token.clone(), token.clone(),
            token.clone(), token.clone(), token,
        ]);
    }

    sql.push_str(
        " ORDER BY CASE status WHEN 'reading' THEN 0 WHEN 'wishlist' THEN 1 WHEN 'read' THEN 2 ELSE 3 END, updated_at DESC",
    );

    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(params_from_iter(values.iter()), row_to_book)
        .map_err(|e| e.to_string())?;

    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

pub fn get_book(id: i64) -> Result<Option<Book>, String> {
    init()?;
    let conn = connect()?;
    let mut stmt = conn.prepare("SELECT * FROM books WHERE id=?1").map_err(|e| e.to_string())?;
    let mut rows = stmt.query(params![id]).map_err(|e| e.to_string())?;
    match rows.next().map_err(|e| e.to_string())? {
        Some(row) => row_to_book(row).map(Some).map_err(|e| e.to_string()),
        None => Ok(None),
    }
}

pub fn save_book(mut book: Book) -> Result<Book, String> {
    init()?;
    let conn = connect()?;

    if book.title.trim().is_empty() {
        return Err("Le titre du livre est obligatoire.".to_string());
    }
    if book.authors.trim().is_empty() {
        book.authors = "Auteur inconnu".to_string();
    }
    if let Some(isbn) = book.isbn.as_ref() {
        let clean = isbn
            .chars()
            .filter(|character| character.is_ascii_digit() || *character == 'X' || *character == 'x')
            .map(|character| character.to_ascii_uppercase())
            .collect::<String>();
        book.isbn = if clean.is_empty() { None } else { Some(clean) };
    }

    let existing_by_isbn = if book.id.is_none() {
        if let Some(isbn) = book.isbn.as_ref() {
            conn.query_row(
                "SELECT id FROM books WHERE isbn=?1",
                params![isbn],
                |row| row.get::<_, i64>(0),
            )
            .ok()
        } else {
            None
        }
    } else {
        None
    };
    if existing_by_isbn.is_some() {
        book.id = existing_by_isbn;
    }

    let owned = if book.owned { 1 } else { 0 };
    if let Some(id) = book.id {
        conn.execute(
            r#"UPDATE books SET
                work_key=?1, edition_key=?2, source=?3, source_id=?4, isbn=?5,
                title=?6, authors=?7, publisher=?8, collection=?9, published_date=?10,
                format=?11, description=?12, subjects=?13, publish_year=?14, pages=?15,
                language=?16, cover_url=?17, cover_path=?18, status=?19, owned=?20,
                rating=?21, review=?22, liked=?23, disliked=?24, location=?25,
                purchase_price=?26, purchase_date=?27, started_at=?28, finished_at=?29,
                progress_pages=?30, loaned_to=?31, tags=?32, updated_at=CURRENT_TIMESTAMP
               WHERE id=?33"#,
            params![
                book.work_key, book.edition_key, book.source, book.source_id, book.isbn,
                book.title, book.authors, book.publisher, book.collection, book.published_date,
                book.format, book.description, book.subjects, book.publish_year, book.pages,
                book.language, book.cover_url, book.cover_path, book.status, owned,
                book.rating, book.review, book.liked, book.disliked, book.location,
                book.purchase_price, book.purchase_date, book.started_at, book.finished_at,
                book.progress_pages, book.loaned_to, book.tags, id
            ],
        )
        .map_err(|error| format!("Impossible d’enregistrer le livre : {error}"))?;
        return get_book(id)?.ok_or_else(|| "Le livre enregistré est introuvable.".to_string());
    }

    conn.execute(
        r#"INSERT INTO books (
            work_key, edition_key, source, source_id, isbn, title, authors, publisher,
            collection, published_date, format, description, subjects, publish_year,
            pages, language, cover_url, cover_path, status, owned, rating, review,
            liked, disliked, location, purchase_price, purchase_date, started_at,
            finished_at, progress_pages, loaned_to, tags
        ) VALUES (
            ?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,
            ?17,?18,?19,?20,?21,?22,?23,?24,?25,?26,?27,?28,?29,?30,?31,?32
        )"#,
        params![
            book.work_key, book.edition_key, book.source, book.source_id, book.isbn,
            book.title, book.authors, book.publisher, book.collection, book.published_date,
            book.format, book.description, book.subjects, book.publish_year, book.pages,
            book.language, book.cover_url, book.cover_path, book.status, owned,
            book.rating, book.review, book.liked, book.disliked, book.location,
            book.purchase_price, book.purchase_date, book.started_at, book.finished_at,
            book.progress_pages, book.loaned_to, book.tags
        ],
    )
    .map_err(|error| format!("Impossible d’ajouter le livre : {error}"))?;
    let id = conn.last_insert_rowid();
    get_book(id)?.ok_or_else(|| "Le livre ajouté est introuvable.".to_string())
}

pub fn delete_book(id: i64) -> Result<(), String> {
    init()?;
    let conn = connect()?;
    conn.execute("DELETE FROM books WHERE id=?1", params![id])
        .map_err(|e| format!("Impossible de supprimer le livre : {e}"))?;
    Ok(())
}

fn fold_char(character: char) -> char {
    match character {
        'à' | 'á' | 'â' | 'ä' | 'ã' | 'å' | 'À' | 'Á' | 'Â' | 'Ä' | 'Ã' | 'Å' => 'a',
        'ç' | 'Ç' => 'c',
        'è' | 'é' | 'ê' | 'ë' | 'È' | 'É' | 'Ê' | 'Ë' => 'e',
        'ì' | 'í' | 'î' | 'ï' | 'Ì' | 'Í' | 'Î' | 'Ï' => 'i',
        'ñ' | 'Ñ' => 'n',
        'ò' | 'ó' | 'ô' | 'ö' | 'õ' | 'Ò' | 'Ó' | 'Ô' | 'Ö' | 'Õ' => 'o',
        'ù' | 'ú' | 'û' | 'ü' | 'Ù' | 'Ú' | 'Û' | 'Ü' => 'u',
        'ý' | 'ÿ' | 'Ý' => 'y',
        'œ' | 'Œ' => 'o',
        _ => character.to_ascii_lowercase(),
    }
}

fn normalized_text(value: &str) -> String {
    value
        .chars()
        .map(fold_char)
        .collect::<String>()
        .split(|character: char| !character.is_ascii_alphanumeric())
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

fn strip_parenthetical(value: &str) -> String {
    let mut result = String::new();
    let mut depth = 0_u8;
    for character in value.chars() {
        match character {
            '(' | '[' | '{' => depth = depth.saturating_add(1),
            ')' | ']' | '}' => depth = depth.saturating_sub(1),
            _ if depth == 0 => result.push(character),
            _ => {}
        }
    }
    result
}

fn clean_author_name(value: &str) -> Option<String> {
    let first = value
        .split(['/', ';', '|'])
        .next()
        .unwrap_or(value)
        .trim();
    if first.is_empty() || first.eq_ignore_ascii_case("Auteur inconnu") {
        return None;
    }

    let stripped = strip_parenthetical(first);
    let without_role = stripped
        .split(". Auteur")
        .next()
        .unwrap_or(&stripped)
        .split(". Autrice")
        .next()
        .unwrap_or(&stripped)
        .trim()
        .trim_matches(|character: char| character == '.' || character == ',')
        .trim();
    if without_role.is_empty() {
        return None;
    }

    let display = if let Some((surname, given)) = without_role.split_once(',') {
        let surname = surname.trim();
        let given = given.trim();
        if given.is_empty() { surname.to_string() } else { format!("{given} {surname}") }
    } else {
        without_role.to_string()
    };

    let display = display.split_whitespace().collect::<Vec<_>>().join(" ");
    if display.is_empty() { None } else { Some(display) }
}

fn top_counts(values: impl Iterator<Item = String>, limit: usize) -> Vec<NamedCount> {
    let mut counts: HashMap<String, (String, i64)> = HashMap::new();
    for value in values {
        let display = value.trim();
        if display.is_empty() {
            continue;
        }
        let key = normalized_text(display);
        if key.is_empty() {
            continue;
        }
        let entry = counts.entry(key).or_insert_with(|| (display.to_string(), 0));
        entry.1 += 1;
    }
    let mut items = counts
        .into_values()
        .map(|(name, count)| NamedCount { name, count })
        .collect::<Vec<_>>();
    items.sort_by(|a, b| b.count.cmp(&a.count).then_with(|| a.name.cmp(&b.name)));
    items.truncate(limit);
    items
}


const CLASSIC_AUTHOR_NAMES: &[&str] = &[
    "alexandre dumas",
    "stefan zweig",
    "albert camus",
    "franz kafka",
    "moliere",
    "madame de la fayette",
    "la fayette",
    "victor hugo",
    "jean de la fontaine",
    "choderlos de laclos",
    "alfred de musset",
    "virgile",
    "eugene ionesco",
    "gaston leroux",
    "george orwell",
    "romain gary",
    "jacques prevert",
    "homere",
    "maurice druon",
    "italo calvino",
];

fn contains_phrase(haystack: &str, phrase: &str) -> bool {
    let haystack = format!(" {} ", normalized_text(haystack));
    let phrase = normalized_text(phrase);
    !phrase.is_empty() && haystack.contains(&format!(" {phrase} "))
}

fn contains_any(haystack: &str, phrases: &[&str]) -> bool {
    phrases.iter().any(|phrase| contains_phrase(haystack, phrase))
}

fn contains_name(value: &str, name: &str) -> bool {
    let normalized_value = normalized_text(value);
    let normalized_name = normalized_text(name);
    let value_tokens = normalized_value
        .split_whitespace()
        .collect::<HashSet<_>>();
    let name_tokens = normalized_name
        .split_whitespace()
        .collect::<Vec<_>>();
    !name_tokens.is_empty()
        && name_tokens
            .iter()
            .all(|token| value_tokens.contains(token))
}

fn contains_any_name(value: &str, names: &[&str]) -> bool {
    names.iter().any(|name| contains_name(value, name))
}

fn theme_labels(book: &Book) -> HashSet<&'static str> {
    let title = normalized_text(&book.title);
    let authors = normalized_text(&book.authors);
    let publisher = normalized_text(&book.publisher);
    let collection = normalized_text(&book.collection);
    let format = normalized_text(&book.format);
    let subjects = normalized_text(&book.subjects);
    let tags = normalized_text(&book.tags);
    let description = normalized_text(&book.description);
    let metadata = format!(
        "{title} {authors} {publisher} {collection} {format} {subjects} {tags} {description}"
    );

    let mut labels = HashSet::new();

    let is_harry_potter = contains_any(
        &title,
        &[
            "harry potter",
            "animaux fantastiques",
            "crimes de grindelwald",
            "contes de beedle",
        ],
    );
    let is_manga = contains_any(
        &metadata,
        &[
            "manga",
            "shonen",
            "seinen",
            "one piece",
            "naruto",
            "tokyo ghoul",
            "dr stone",
            "pokemon",
        ],
    ) || contains_any(&publisher, &["kurokawa", "kana"]);

    if is_manga {
        labels.insert("Manga");
    }

    if contains_any(
        &metadata,
        &[
            "young adult",
            "juvenile fiction",
            "juvenile literature",
            "children s fiction",
        ],
    ) || contains_any(&collection, &["folio junior", "pocket jeunesse"])
        || contains_any(
            &publisher,
            &["pocket jeunesse", "slalom", "gallimard jeunesse"],
        )
        || contains_any(
            &title,
            &[
                "hunger games",
                "labyrinthe",
                "frigiel",
                "harry potter",
                "nicolas flamel",
            ],
        )
    {
        labels.insert("Jeunesse");
    }

    if contains_any(
        &collection,
        &[
            "classique",
            "classiques",
            "classico",
            "classicolycee",
            "classiques cie",
            "etonnants classiques",
            "carres classiques",
            "folio classique",
        ],
    ) || contains_any_name(&authors, CLASSIC_AUTHOR_NAMES)
    {
        labels.insert("Classiques");
    }

    if contains_any(
        &format!("{subjects} {tags}"),
        &[
            "adventure",
            "aventure",
            "pirates",
            "quest",
            "action and adventure",
        ],
    ) || contains_any(
        &title,
        &[
            "one piece",
            "naruto",
            "dr stone",
            "assassin s creed",
            "hunger games",
            "labyrinthe",
            "comte de monte cristo",
            "trois mousquetaires",
            "seigneur des anneaux",
            "fraternite de l anneau",
            "deux tours",
            "retour du roi",
            "zelda",
            "frigiel",
            "nicolas flamel",
            "grimoire au rubis",
            "endgame",
        ],
    ) {
        labels.insert("Aventure");
    }

    if contains_any(
        &subjects,
        &[
            "fantasy fiction",
            "fantasy",
            "magic",
            "wizards",
            "witches",
            "supernatural",
        ],
    ) || contains_any(
        &title,
        &[
            "harry potter",
            "animaux fantastiques",
            "crimes de grindelwald",
            "seigneur des anneaux",
            "fraternite de l anneau",
            "deux tours",
            "retour du roi",
            "zelda",
            "frigiel",
            "nicolas flamel",
            "grimoire au rubis",
        ],
    ) {
        labels.insert("Fantastique et fantasy");
    }

    if contains_any(&subjects, &["horror", "gothic"])
        || contains_any(
            &title,
            &["tokyo ghoul", "fantome de l opera", "metamorphose"],
        )
    {
        labels.insert("Horreur et gothique");
    }

    if contains_any(
        &title,
        &[
            "1984",
            "ferme des animaux",
            "hunger games",
            "labyrinthe",
            "rhinoceros",
        ],
    ) || contains_any(
        &subjects,
        &["dystopian", "totalitarian", "surveillance society"],
    ) {
        labels.insert("Dystopie");
    }

    if contains_any(
        &title,
        &[
            "fouche",
            "rois maudits",
            "renaissance",
            "brotherhood",
            "croisade secrete",
            "revelations",
            "black flag",
            "unity",
            "underworld",
            "forsaken",
            "comte de monte cristo",
            "trois mousquetaires",
            "pauline",
        ],
    ) || contains_any(&subjects, &["historical fiction"])
    {
        labels.insert("Roman historique et histoire");
    }

    // « Philosopher's Stone » est un titre de fantasy, pas un ouvrage de
    // philosophie. On ne classe donc pas Harry Potter avec les essais.
    if !is_harry_potter
        && (contains_any_name(&authors, &["albert camus", "hannah arendt", "simone weil"])
            || contains_any(
                &subjects,
                &["philosophy", "existentialism", "absurd"],
            )
            || contains_any(
                &title,
                &[
                    "etranger",
                    "crise de la culture",
                    "verite et politique",
                    "mensonge a la violence",
                    "condition ouvriere",
                ],
            ))
    {
        labels.insert("Philosophie");
    }

    if contains_any_name(
        &authors,
        &["george orwell", "hannah arendt", "simone weil"],
    ) || contains_any(
        &subjects,
        &[
            "politics",
            "political violence",
            "civil disobedience",
            "social conditions",
            "sociology",
        ],
    ) || contains_any(
        &title,
        &[
            "1984",
            "ferme des animaux",
            "crise de la culture",
            "verite et politique",
            "mensonge a la violence",
            "condition ouvriere",
        ],
    ) {
        labels.insert("Politique et société");
    }

    if contains_any(&format, &["theatre", "piece de theatre"])
        || contains_any(
            &title,
            &[
                "malade imaginaire",
                "lorenzaccio",
                "rhinoceros",
                "par dessus bord",
                "enfant maudit",
            ],
        )
        || contains_any_name(
            &authors,
            &["moliere", "alfred de musset", "eugene ionesco", "michel vinaver"],
        )
    {
        labels.insert("Théâtre");
    }

    if contains_any(&title, &["contemplations", "paroles", "georgiques"])
        || contains_any(&subjects, &["poetry", "poesie"])
        || contains_any_name(&authors, &["jacques prevert", "virgile"])
    {
        labels.insert("Poésie");
    }

    if contains_any(&title, &["iliade", "odyssee"])
        || contains_any_name(&authors, &["homere"])
        || contains_any(&subjects, &["mythology", "epic poetry"])
    {
        labels.insert("Mythologie et épopée");
    }

    if contains_any(&title, &["petite histoire de l art"])
        || contains_any(&subjects, &["histoire de l art", "art history"])
    {
        labels.insert("Art");
    }

    if contains_any(&title, &["tout l or du monde"])
        || contains_any(
            &subjects,
            &["economie", "sociologie economique", "market economy"],
        )
    {
        labels.insert("Économie");
    }

    if contains_any(&title, &["bible"])
        || contains_any(&subjects, &["religion", "christianity", "theology"])
    {
        labels.insert("Religion et spiritualité");
    }

    if contains_any(&title, &["art de la cuisine"])
        || contains_any(&subjects, &["cookery", "cooking"])
    {
        labels.insert("Cuisine");
    }

    if contains_any(
        &title,
        &["pitie dangereuse", "lettre d une inconnue", "amok"],
    ) || contains_any(
        &subjects,
        &["psychological fiction", "psychology"],
    ) {
        labels.insert("Psychologie");
    }

    if contains_any(&title, &["breviaire des echecs"]) {
        labels.insert("Jeux et stratégie");
    }

    // Dr. Stone peut porter le thème scientifique, mais le mot « science »
    // présent dans une notice Harry Potter ne suffit jamais à le classer ici.
    if contains_any(&title, &["dr stone"])
        || (!is_harry_potter
            && contains_any(
                &subjects,
                &["natural sciences", "biology", "physics", "chemistry", "astronomy"],
            ))
    {
        labels.insert("Sciences");
    }

    labels
}

fn theme_counts(books: &[&Book], limit: usize) -> Vec<NamedCount> {
    let mut counts: HashMap<&'static str, i64> = HashMap::new();

    for book in books {
        for label in theme_labels(book) {
            *counts.entry(label).or_insert(0) += 1;
        }
    }

    let mut items = counts
        .into_iter()
        .map(|(name, count)| NamedCount {
            name: name.to_string(),
            count,
        })
        .collect::<Vec<_>>();
    items.sort_by(|a, b| b.count.cmp(&a.count).then_with(|| a.name.cmp(&b.name)));
    items.truncate(limit);
    items
}

pub fn stats() -> Result<DashboardStats, String> {
    let books = list_books(Some("all"), None)?;
    let rated = books.iter().filter_map(|b| b.rating).collect::<Vec<_>>();
    let average_rating = if rated.is_empty() {
        0.0
    } else {
        ((rated.iter().sum::<f64>() / rated.len() as f64) * 10.0).round() / 10.0
    };

    // Les thèmes décrivent la bibliothèque physique, pas seulement les livres
    // dont le statut est déjà « lu ». Plusieurs imports historiques ont laissé
    // des exemplaires possédés avec le statut « wishlist » ; ils doivent tout
    // de même compter dans la composition de la collection.
    let profile_books = books
        .iter()
        .filter(|book| book.owned)
        .collect::<Vec<_>>();

    let top_authors = top_counts(
        books
            .iter()
            .filter(|book| book.rating.unwrap_or(0.0) >= 6.0)
            .filter_map(|book| clean_author_name(&book.authors)),
        6,
    );
    let top_subjects = theme_counts(&profile_books, 24);

    Ok(DashboardStats {
        total: books.len() as i64,
        owned: books.iter().filter(|b| b.owned).count() as i64,
        read: books.iter().filter(|b| b.status == "read").count() as i64,
        reading: books.iter().filter(|b| b.status == "reading").count() as i64,
        wishlist: books.iter().filter(|b| b.status == "wishlist").count() as i64,
        abandoned: books.iter().filter(|b| b.status == "abandoned").count() as i64,
        average_rating,
        pages_read: books
            .iter()
            .filter(|b| b.status == "read")
            .map(|b| b.pages.unwrap_or(0))
            .sum(),
        profiled_books: profile_books.len() as i64,
        top_authors,
        top_subjects,
    })
}

pub fn save_feedback(candidate_key: &str, action: &str) -> Result<(), String> {
    init()?;
    let conn = connect()?;
    conn.execute(
        "INSERT INTO recommendation_feedback(candidate_key, action) VALUES (?1, ?2)",
        params![candidate_key, action],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn feedback() -> Result<HashMap<String, String>, String> {
    init()?;
    let conn = connect()?;
    let mut stmt = conn
        .prepare("SELECT candidate_key, action FROM recommendation_feedback ORDER BY created_at")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)))
        .map_err(|e| e.to_string())?;
    let mut result = HashMap::new();
    for row in rows {
        let (key, action) = row.map_err(|e| e.to_string())?;
        result.insert(key, action);
    }
    Ok(result)
}

pub fn export_json(path: &Path) -> Result<usize, String> {
    let books = list_books(Some("all"), None)?;
    let payload = serde_json::to_string_pretty(&books).map_err(|e| e.to_string())?;
    fs::write(path, payload).map_err(|e| format!("Impossible d’exporter les données : {e}"))?;
    Ok(books.len())
}

pub fn import_json(path: &Path) -> Result<usize, String> {
    let payload = fs::read_to_string(path).map_err(|e| format!("Impossible de lire l’export : {e}"))?;
    let mut books: Vec<Book> = serde_json::from_str(&payload).map_err(|e| format!("Export Libris invalide : {e}"))?;
    let mut count = 0;
    for book in &mut books {
        book.id = None;
        save_book(book.clone())?;
        count += 1;
    }
    Ok(count)
}
