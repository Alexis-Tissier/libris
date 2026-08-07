use crate::db;
use crate::models::{Book, Recommendation, SearchResult};
use crate::openlibrary;
use std::collections::{HashMap, HashSet};

const STOPWORDS: &[&str] = &[
    "a", "an", "and", "author", "avec", "book", "books", "children", "d", "dans",
    "de", "des", "du", "elle", "elles", "en", "entre", "est", "et", "fiction", "for",
    "from", "general", "into", "juvenile", "la", "language", "le", "les", "leur", "leurs",
    "literary", "litterature", "mais", "novel", "of", "pour", "que", "qui", "roman",
    "series", "son", "stories", "story", "sur", "the", "to", "translated", "un", "une",
    "work", "works", "l",
];

const TITLE_NOISE: &[&str] = &[
    "abrege", "adaptation", "analyse", "analysis", "avec dossier", "commentaire", "companion",
    "critique", "edition", "etude", "extraits", "fiche de lecture", "guide de lecture",
    "reader guide", "resume", "study guide", "texte integral",
];

const ROLE_WORDS: &[&str] = &[
    "auteur", "autrice", "texte", "traduit", "traduction", "edition", "editeur", "preface",
    "dossier", "notes", "chronologie", "bibliographie",
];

const TITLE_DESCRIPTOR_WORDS: &[&str] = &[
    "tome", "volume", "vol", "numero", "no", "texte", "film", "integral",
    "integrale", "edition", "ed", "illustre", "illustree",
];

const HARRY_POTTER_ALIASES: &[(u16, &[&str])] = &[
    (1, &["ecole des sorciers", "philosopher s stone", "sorcerer s stone"]),
    (2, &["chambre des secrets", "chamber of secrets"]),
    (3, &["prisonnier d azkaban", "prisoner of azkaban"]),
    (4, &["coupe de feu", "goblet of fire"]),
    (5, &["ordre du phenix", "order of the phoenix"]),
    (6, &["prince de sang mele", "half blood prince"]),
    (7, &["reliques de la mort", "deathly hallows"]),
];

const LORD_OF_THE_RINGS_ALIASES: &[(u16, &[&str])] = &[
    (1, &["fraternite de l anneau", "fellowship of the ring"]),
    (2, &["deux tours", "two towers"]),
    (3, &["retour du roi", "return of the king"]),
];

const RECOMMENDATION_GENRES: &[(&str, &str, &str, &[&str])] = &[
    ("adventure", "Aventure", "adventure fiction", &["adventure", "aventure", "pirate", "quest", "exploration"]),
    ("fantasy", "Fantastique et fantasy", "fantasy", &["fantasy", "fantastique", "magic", "sorcellerie", "dragon"]),
    ("classics", "Romans classiques", "classic literature", &["classic", "classique", "literary fiction"]),
    ("dystopia", "Dystopie", "dystopian fiction", &["dystop", "totalitarian", "surveillance"]),
    ("science-fiction", "Science-fiction", "science fiction", &["science fiction", "sci fi", "anticipation"]),
    ("mystery", "Mystère et enquête", "mystery fiction", &["mystery", "detective", "enquete", "crime", "thriller"]),
    ("horror", "Horreur et gothique", "horror fiction", &["horror", "horreur", "gothic", "gothique"]),
    ("history", "Roman historique et histoire", "historical fiction", &["historical", "histoire", "history", "antiquity"]),
    ("philosophy", "Philosophie et société", "philosophy", &["philosoph", "politic", "society", "sociologie", "existential"]),
    ("theatre-poetry", "Théâtre et poésie", "drama", &["drama", "theatre", "poetry", "poesie"]),
    ("manga", "Manga", "manga", &["manga", "shonen", "seinen"]),
    ("mythology", "Mythologie et épopée", "mythology", &["mytholog", "epic", "epopee", "homere"]),
];

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

fn normalized_words(value: &str) -> Vec<String> {
    value
        .chars()
        .map(fold_char)
        .collect::<String>()
        .split(|character: char| !character.is_ascii_alphanumeric())
        .map(str::trim)
        .filter(|token| !token.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

fn strip_bracketed(value: &str) -> String {
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

fn author_words(value: &str) -> Vec<String> {
    let first = value
        .split(['/', ';', '|'])
        .next()
        .unwrap_or(value);
    normalized_words(&strip_bracketed(first))
        .into_iter()
        .filter(|token| token.len() > 1)
        .filter(|token| !ROLE_WORDS.contains(&token.as_str()))
        .filter(|token| !STOPWORDS.contains(&token.as_str()))
        .filter(|token| !token.chars().all(|character| character.is_ascii_digit()))
        .collect()
}

fn author_query(value: &str) -> String {
    let words = author_words(value);
    if words.is_empty() {
        return String::new();
    }
    words.into_iter().take(4).collect::<Vec<_>>().join(" ")
}

fn author_key(value: &str) -> String {
    let mut words = author_words(value);
    words.sort();
    words.dedup();
    words.join(" ")
}

fn author_surname(value: &str) -> String {
    let first = value.split(['/', ';', '|']).next().unwrap_or(value);
    if let Some((surname, _)) = first.split_once(',') {
        return normalized_words(surname)
            .into_iter()
            .filter(|token| token.len() > 1)
            .last()
            .unwrap_or_default();
    }
    author_words(first).last().cloned().unwrap_or_default()
}

fn title_words(title: &str, authors: &str) -> Vec<String> {
    let first_segment = title.split(['/', ';']).next().unwrap_or(title);
    let unbracketed = strip_bracketed(first_segment);
    let author_tokens = author_words(authors).into_iter().collect::<HashSet<_>>();
    let words = normalized_words(&unbracketed);
    let mut cleaned = Vec::new();
    let mut skip_volume_number = false;

    for token in words {
        if matches!(token.as_str(), "tome" | "volume" | "vol" | "numero" | "no") {
            skip_volume_number = true;
            continue;
        }
        if skip_volume_number && token.chars().all(|character| character.is_ascii_digit()) {
            skip_volume_number = false;
            continue;
        }
        skip_volume_number = false;

        if STOPWORDS.contains(&token.as_str())
            || ROLE_WORDS.contains(&token.as_str())
            || TITLE_DESCRIPTOR_WORDS.contains(&token.as_str())
        {
            continue;
        }
        cleaned.push(token);
    }

    if let Some(position) = cleaned.iter().position(|word| word == "par" || word == "by") {
        let suffix = cleaned[position + 1..].iter().collect::<HashSet<_>>();
        if suffix.iter().any(|word| author_tokens.contains(*word)) {
            cleaned.truncate(position);
        }
    }

    // Certaines notices répètent la série avant le titre réel :
    // « Harry Potter, tome 5, Harry Potter et l’Ordre du Phénix ».
    let mut seen = HashSet::new();
    cleaned.retain(|token| seen.insert(token.clone()));
    cleaned
}

fn canonical_title(title: &str, authors: &str) -> String {
    title_words(title, authors).join(" ")
}

fn title_set(title: &str, authors: &str) -> HashSet<String> {
    title_words(title, authors).into_iter().collect()
}

fn explicit_volume(words: &[String]) -> Option<u16> {
    for pair in words.windows(2) {
        if matches!(pair[0].as_str(), "tome" | "volume" | "vol" | "book") {
            if let Ok(number) = pair[1].parse::<u16>() {
                return Some(number);
            }
        }
    }
    None
}

fn contains_phrase(value: &str, phrase: &str) -> bool {
    value == phrase
        || value.starts_with(&format!("{phrase} "))
        || value.ends_with(&format!(" {phrase}"))
        || value.contains(&format!(" {phrase} "))
}

fn series_work_identity(title: &str) -> Option<String> {
    let words = normalized_words(title);
    let value = words.join(" ");

    if contains_phrase(&value, "harry potter") {
        let volume = explicit_volume(&words).or_else(|| {
            HARRY_POTTER_ALIASES.iter().find_map(|(number, aliases)| {
                aliases
                    .iter()
                    .any(|alias| contains_phrase(&value, alias))
                    .then_some(*number)
            })
        });
        return volume.map(|number| format!("harry-potter:{number}"));
    }

    for (needle, key) in [
        ("one piece", "one-piece"),
        ("naruto", "naruto"),
        ("tokyo ghoul", "tokyo-ghoul"),
        ("dr stone", "dr-stone"),
        ("pokemon", "pokemon"),
    ] {
        if contains_phrase(&value, needle) {
            if let Some(number) = explicit_volume(&words) {
                return Some(format!("{key}:{number}"));
            }
        }
    }

    if contains_phrase(&value, "seigneur des anneaux")
        || contains_phrase(&value, "lord of the rings")
    {
        let volume = LORD_OF_THE_RINGS_ALIASES
            .iter()
            .find_map(|(number, aliases)| {
                aliases
                    .iter()
                    .any(|alias| contains_phrase(&value, alias))
                    .then_some(*number)
            })
            .or_else(|| explicit_volume(&words));
        return volume.map(|number| format!("lord-of-the-rings:{number}"));
    }

    None
}

fn normalized_isbn(value: &str) -> String {
    value
        .chars()
        .filter(|character| character.is_ascii_digit() || *character == 'X' || *character == 'x')
        .map(|character| character.to_ascii_uppercase())
        .collect()
}

fn meaningful_tokens(value: &str) -> HashSet<String> {
    normalized_words(value)
        .into_iter()
        .filter(|token| token.len() >= 4)
        .filter(|token| !STOPWORDS.contains(&token.as_str()))
        .filter(|token| !ROLE_WORDS.contains(&token.as_str()))
        .collect()
}

fn book_tokens(book: &Book) -> HashSet<String> {
    meaningful_tokens(&format!(
        "{} {} {} {} {}",
        book.subjects, book.description, book.liked, book.tags, book.title
    ))
}

fn result_tokens(book: &SearchResult) -> HashSet<String> {
    meaningful_tokens(&format!(
        "{} {} {}",
        book.subjects, book.description, book.title
    ))
}

fn similarity(first: &HashSet<String>, second: &HashSet<String>) -> f64 {
    if first.is_empty() || second.is_empty() {
        return 0.0;
    }
    let common = first.intersection(second).count();
    // Un seul mot générique ne suffit plus à déclarer deux livres proches.
    // C’était notamment la cause de rapprochements absurdes avec One Piece.
    if common < 2 {
        return 0.0;
    }
    let denominator = ((first.len() * second.len()) as f64).sqrt();
    if denominator == 0.0 {
        0.0
    } else {
        common as f64 / denominator
    }
}

fn authors_overlap(first: &str, second: &str) -> bool {
    let first = author_words(first).into_iter().collect::<HashSet<_>>();
    let second = author_words(second).into_iter().collect::<HashSet<_>>();
    !first.is_empty() && !second.is_empty() && first.intersection(&second).count() >= 1
}

fn candidate_isbns(candidate: &SearchResult) -> HashSet<String> {
    candidate
        .isbn
        .iter()
        .chain(candidate.alternate_isbns.iter())
        .map(|value| normalized_isbn(value))
        .filter(|value| !value.is_empty())
        .collect()
}

fn same_work_with_isbns(
    book: &Book,
    candidate: &SearchResult,
    all_candidate_isbns: &HashSet<String>,
) -> bool {
    if book
        .work_key
        .as_ref()
        .is_some_and(|key| !key.is_empty() && key == &candidate.work_key)
    {
        return true;
    }

    if let Some(isbn) = book.isbn.as_ref() {
        let isbn = normalized_isbn(isbn);
        if !isbn.is_empty() && all_candidate_isbns.contains(&isbn) {
            return true;
        }
    }

    if let (Some(first), Some(second)) = (
        series_work_identity(&book.title),
        series_work_identity(&candidate.title),
    ) {
        if first == second {
            return true;
        }
    }

    let first_title = canonical_title(&book.title, &book.authors);
    let second_title = canonical_title(&candidate.title, &candidate.authors);
    if first_title.is_empty() || second_title.is_empty() {
        return false;
    }
    if first_title == second_title && authors_overlap(&book.authors, &candidate.authors) {
        return true;
    }
    if !authors_overlap(&book.authors, &candidate.authors) {
        return false;
    }

    let first_set = title_set(&book.title, &book.authors);
    let second_set = title_set(&candidate.title, &candidate.authors);
    let shorter = first_set.len().min(second_set.len());
    if shorter == 0 {
        return false;
    }
    let overlap = first_set.intersection(&second_set).count();
    let containment = overlap as f64 / shorter as f64;

    containment >= 0.9
        && (shorter >= 3
            || (shorter >= 2 && first_set.len().abs_diff(second_set.len()) <= 2))
}

#[cfg(test)]
fn same_work(book: &Book, candidate: &SearchResult) -> bool {
    let all_candidate_isbns = candidate_isbns(candidate);
    same_work_with_isbns(book, candidate, &all_candidate_isbns)
}

fn same_library_work(first: &Book, second: &Book) -> bool {
    if let (Some(first_key), Some(second_key)) = (&first.work_key, &second.work_key) {
        if !first_key.is_empty() && first_key == second_key {
            return true;
        }
    }

    if let (Some(first_isbn), Some(second_isbn)) = (&first.isbn, &second.isbn) {
        if normalized_isbn(first_isbn) == normalized_isbn(second_isbn) {
            return true;
        }
    }

    if let (Some(first_series), Some(second_series)) = (
        series_work_identity(&first.title),
        series_work_identity(&second.title),
    ) {
        if first_series == second_series {
            return true;
        }
    }

    if !authors_overlap(&first.authors, &second.authors) {
        return false;
    }

    let first_set = title_set(&first.title, &first.authors);
    let second_set = title_set(&second.title, &second.authors);
    let shorter = first_set.len().min(second_set.len());
    if shorter == 0 {
        return false;
    }
    let overlap = first_set.intersection(&second_set).count();
    let containment = overlap as f64 / shorter as f64;
    containment >= 0.9
        && (shorter >= 3
            || (shorter >= 2 && first_set.len().abs_diff(second_set.len()) <= 2))
}

fn is_french(candidate: &SearchResult) -> bool {
    let language = candidate.language.trim().to_lowercase();
    matches!(language.as_str(), "fr" | "fre" | "fra" | "french")
}

fn is_low_quality(candidate: &SearchResult, known_authors: &HashSet<String>) -> bool {
    let title = normalized_words(&candidate.title).join(" ");
    if title.is_empty() || candidate.authors.trim().is_empty() || candidate.authors == "Auteur inconnu" {
        return true;
    }
    if TITLE_NOISE.iter().any(|noise| title.contains(noise)) {
        return true;
    }

    let canonical = canonical_title(&candidate.title, &candidate.authors);
    let surname = author_surname(&candidate.authors);
    if canonical.split_whitespace().count() <= 2
        && (known_authors.contains(&canonical) || (!surname.is_empty() && canonical == surname))
    {
        return true;
    }

    false
}

fn genre_definition(key: &str) -> Option<(&'static str, &'static str, &'static [&'static str])> {
    RECOMMENDATION_GENRES
        .iter()
        .find(|(candidate_key, _, _, _)| *candidate_key == key)
        .map(|(_, label, subject, keywords)| (*label, *subject, *keywords))
}

fn inferred_genre_queries(favorites: &[&Book]) -> Vec<String> {
    let profile = favorites
        .iter()
        .map(|book| format!("{} {} {} {} {}", book.subjects, book.tags, book.liked, book.description, book.title))
        .collect::<Vec<_>>()
        .join(" ");
    let normalized = normalized_words(&profile).join(" ");
    let mut matches = RECOMMENDATION_GENRES
        .iter()
        .map(|(_, _, subject, keywords)| {
            let count = keywords
                .iter()
                .filter(|keyword| normalized.contains(&normalized_words(keyword).join(" ")))
                .count();
            (count, *subject)
        })
        .filter(|(count, _)| *count > 0)
        .collect::<Vec<_>>();
    matches.sort_by(|first, second| second.0.cmp(&first.0));
    matches
        .into_iter()
        .take(4)
        .map(|(_, subject)| format!("subject:\"{subject}\" language:fre"))
        .collect()
}

const CLASSIC_AUTHORS: &[&str] = &[
    "alexandre dumas",
    "stefan zweig",
    "albert camus",
    "franz kafka",
    "moliere",
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
    "madame de la fayette",
];

fn has_any_phrase(value: &str, phrases: &[&str]) -> bool {
    let value = normalized_words(value).join(" ");
    phrases.iter().any(|phrase| {
        let phrase = normalized_words(phrase).join(" ");
        !phrase.is_empty() && contains_phrase(&value, &phrase)
    })
}

fn has_name(value: &str, name: &str) -> bool {
    let value_tokens = normalized_words(value).into_iter().collect::<HashSet<_>>();
    let name_tokens = normalized_words(name);
    !name_tokens.is_empty()
        && name_tokens
            .iter()
            .all(|token| value_tokens.contains(token))
}

fn has_any_name(value: &str, names: &[&str]) -> bool {
    names.iter().any(|name| has_name(value, name))
}

fn classic_author(authors: &str) -> bool {
    has_any_name(authors, CLASSIC_AUTHORS)
}

fn genre_matches_fields(
    title: &str,
    authors: &str,
    subjects: &str,
    description: &str,
    collection: &str,
    publisher: &str,
    key: &str,
) -> bool {
    let metadata = format!("{title} {authors} {subjects} {description} {collection} {publisher}");
    let is_harry_potter = has_any_phrase(
        title,
        &["harry potter", "animaux fantastiques", "fantastic beasts", "grindelwald"],
    );

    match key {
        "adventure" => has_any_phrase(
            &metadata,
            &[
                "adventure", "aventure", "pirate", "quest", "exploration", "one piece",
                "naruto", "dr stone", "assassin s creed", "hunger games", "labyrinthe",
                "comte de monte cristo", "trois mousquetaires", "seigneur des anneaux",
                "lord of the rings", "zelda", "frigiel", "nicolas flamel",
            ],
        ),
        "fantasy" => has_any_phrase(
            &metadata,
            &[
                "fantasy", "fantastique", "magic", "wizard", "witch", "sorcellerie",
                "dragon", "harry potter", "animaux fantastiques", "fantastic beasts",
                "seigneur des anneaux", "lord of the rings", "zelda", "frigiel",
                "nicolas flamel",
            ],
        ),
        "classics" => {
            classic_author(authors)
                || has_any_phrase(
                    collection,
                    &[
                        "classique", "classiques", "classico", "classicolycee",
                        "classiques cie", "etonnants classiques", "carres classiques",
                        "folio classique",
                    ],
                )
                || has_any_phrase(subjects, &["classic literature", "classics"])
        }
        "dystopia" => has_any_phrase(
            &metadata,
            &[
                "dystopia", "dystopian", "dystopie", "totalitarian", "surveillance society", "1984",
                "ferme des animaux", "animal farm", "hunger games", "labyrinthe",
            ],
        ),
        "science-fiction" => has_any_phrase(
            &metadata,
            &["science fiction", "sci fi", "anticipation", "scientific fiction"],
        ) && !is_harry_potter,
        "mystery" => has_any_phrase(
            &metadata,
            &["mystery", "detective", "enquete", "crime fiction", "thriller", "policier"],
        ),
        "horror" => has_any_phrase(
            &metadata,
            &["horror", "horreur", "gothic", "gothique", "tokyo ghoul", "fantome de l opera"],
        ),
        "history" => has_any_phrase(
            &metadata,
            &[
                "historical fiction", "roman historique", "fouche", "rois maudits",
                "renaissance", "brotherhood", "croisade secrete", "revelations",
                "black flag", "unity", "underworld", "forsaken", "comte de monte cristo",
                "trois mousquetaires", "reine margot", "guerre de troie",
            ],
        ) || has_any_name(authors, &["alexandre dumas", "maurice druon"]),
        "philosophy" => !is_harry_potter
            && (has_any_phrase(
                &metadata,
                &[
                    "philosophy", "philosophical", "philosophie", "existential",
                    "existentialism", "absurd", "ethique", "morale",
                    "etranger", "crise de la culture", "verite et politique",
                    "condition ouvriere",
                ],
            ) || has_any_name(authors, &["albert camus", "hannah arendt", "simone weil"])),
        "theatre-poetry" => has_any_phrase(
            &metadata,
            &[
                "drama", "theatre", "piece de theatre", "poetry", "poesie", "poeme",
                "moliere", "musset", "ionesco", "vinaver", "prevert", "contemplations",
            ],
        ),
        "manga" => has_any_phrase(
            &metadata,
            &["manga", "shonen", "seinen", "one piece", "naruto", "tokyo ghoul", "dr stone", "pokemon"],
        ),
        "mythology" => has_any_phrase(
            &metadata,
            &["mythology", "mythologie", "epic poetry", "epopee", "homere", "iliade", "odyssee"],
        ),
        _ => false,
    }
}

fn genre_matches(candidate: &SearchResult, key: &str) -> bool {
    genre_matches_fields(
        &candidate.title,
        &candidate.authors,
        &candidate.subjects,
        &candidate.description,
        &candidate.collection,
        &candidate.publisher,
        key,
    )
}

fn book_matches_genre(book: &Book, key: &str) -> bool {
    genre_matches_fields(
        &book.title,
        &book.authors,
        &book.subjects,
        &book.description,
        &book.collection,
        &book.publisher,
        key,
    )
}

fn taste_group_key(book: &Book) -> String {
    if let Some(series) = series_work_identity(&book.title) {
        return series.split(':').next().unwrap_or(&series).to_string();
    }
    let author = author_key(&book.authors);
    if !author.is_empty() {
        return format!("author:{author}");
    }
    format!("title:{}", canonical_title(&book.title, &book.authors))
}

pub async fn recommend(
    mood: &str,
    max_pages: Option<i64>,
    genre: &str,
    requested_limit: Option<usize>,
) -> Result<Vec<Recommendation>, String> {
    let output_limit = requested_limit.unwrap_or(24).clamp(12, 36);
    let selected_genre = genre_definition(genre);
    let library = db::list_books(Some("all"), None)?;
    let feedback = db::feedback()?;

    let mut rated = library
        .iter()
        .filter(|book| book.rating.unwrap_or(0.0) >= 7.0)
        .collect::<Vec<_>>();
    rated.sort_by(|first, second| {
        second
            .rating
            .unwrap_or(0.0)
            .total_cmp(&first.rating.unwrap_or(0.0))
            .then_with(|| second.updated_at.cmp(&first.updated_at))
    });

    // Une saga très longue ne doit pas écraser tout le profil : Harry Potter,
    // One Piece ou Naruto ne comptent chacun que comme un signal de goût pour
    // choisir les requêtes, même si chaque tome reste distinct dans SQLite.
    let mut favorites: Vec<&Book> = Vec::new();
    let mut seen_tastes = HashSet::new();
    for book in rated {
        let group = taste_group_key(book);
        if group.is_empty() || !seen_tastes.insert(group) {
            continue;
        }
        favorites.push(book);
        if favorites.len() >= 20 {
            break;
        }
    }

    if favorites.is_empty() {
        return Ok(Vec::new());
    }

    let mut queries = Vec::new();
    if let Some((_, subject, _)) = selected_genre {
        queries.push(format!("subject:\"{subject}\" language:fre"));
    }
    if !mood.trim().is_empty() {
        queries.push(format!("{} language:fre", mood.trim()));
    }

    // Lorsqu’un genre est choisi, les auteurs interrogés viennent uniquement
    // des livres de ce genre déjà présents dans la bibliothèque. Ainsi une
    // recherche « Classiques » s’appuie sur Dumas, Zweig, Camus, etc., et non
    // sur J. K. Rowling simplement parce que Harry Potter est très bien noté.
    let mut genre_anchors = library
        .iter()
        .filter(|book| {
            selected_genre.is_some()
                && book.owned
                && book_matches_genre(book, genre)
                && (matches!(book.status.as_str(), "read" | "reading")
                    || book.rating.unwrap_or(0.0) >= 6.0)
        })
        .collect::<Vec<_>>();
    genre_anchors.sort_by(|first, second| {
        second
            .rating
            .unwrap_or(0.0)
            .total_cmp(&first.rating.unwrap_or(0.0))
            .then_with(|| second.updated_at.cmp(&first.updated_at))
    });

    let author_sources: Vec<&Book> = if selected_genre.is_some() && !genre_anchors.is_empty() {
        genre_anchors
    } else {
        favorites.clone()
    };
    let comparison_sources = author_sources.clone();

    let mut queried_authors = HashSet::new();
    for book in &author_sources {
        let query_author = author_query(&book.authors);
        let key = author_key(&book.authors);
        if !query_author.is_empty() && queried_authors.insert(key) {
            queries.push(format!("author:\"{query_author}\" language:fre"));
        }
        if queried_authors.len() >= 8 {
            break;
        }
    }
    if selected_genre.is_none() {
        queries.extend(inferred_genre_queries(&favorites));
    }
    queries.truncate(12);

    let mut candidates = Vec::new();
    let mut source_seen = HashSet::new();
    for query in queries {
        if let Ok(results) = openlibrary::search(&query, 45).await {
            for result in results {
                let source_key = if !result.work_key.is_empty() {
                    result.work_key.clone()
                } else {
                    format!(
                        "{}::{}",
                        canonical_title(&result.title, &result.authors),
                        author_key(&result.authors)
                    )
                };
                if source_seen.insert(source_key) {
                    candidates.push(result);
                }
            }
        }
    }

    let known_authors = library
        .iter()
        .flat_map(|book| {
            let full = author_key(&book.authors);
            let surname = author_surname(&book.authors);
            [full, surname]
        })
        .filter(|value| !value.is_empty())
        .collect::<HashSet<_>>();
    let favorite_author_keys = comparison_sources
        .iter()
        .map(|book| author_key(&book.authors))
        .filter(|value| !value.is_empty())
        .collect::<HashSet<_>>();
    let mood_tokens = meaningful_tokens(mood);

    let mut scored = Vec::new();
    for candidate in candidates {
        let feedback_key = if !candidate.work_key.is_empty() {
            candidate.work_key.clone()
        } else {
            candidate
                .isbn
                .clone()
                .unwrap_or_else(|| format!("{}::{}", candidate.title, candidate.authors))
        };
        if feedback.get(&feedback_key).is_some_and(|action| action == "dismiss") {
            continue;
        }
        let all_candidate_isbns = candidate_isbns(&candidate);
        if !is_french(&candidate)
            || is_low_quality(&candidate, &known_authors)
            || (selected_genre.is_some() && !genre_matches(&candidate, genre))
            || library
                .iter()
                .any(|book| same_work_with_isbns(book, &candidate, &all_candidate_isbns))
        {
            continue;
        }

        let candidate_tokens = result_tokens(&candidate);
        let candidate_author_key = author_key(&candidate.authors);
        let same_author = favorite_author_keys.contains(&candidate_author_key);

        let mut nearest: Option<(&Book, f64)> = None;
        for favorite in &comparison_sources {
            let similarity = similarity(&book_tokens(favorite), &candidate_tokens);
            if nearest.as_ref().map_or(true, |(_, current)| similarity > *current) {
                nearest = Some((*favorite, similarity));
            }
        }
        let (nearest_book, thematic_similarity) = nearest.unwrap_or((comparison_sources[0], 0.0));
        let mood_similarity = similarity(&mood_tokens, &candidate_tokens);

        let mut score = 25.0 + thematic_similarity * 42.0 + mood_similarity * 12.0;
        let mut reasons = Vec::new();
        let mut warnings = Vec::new();

        if let Some((label, _, _)) = selected_genre {
            score += 28.0;
            reasons.push(format!("genre demandé : {label}"));
        }

        if same_author {
            score += 14.0;
            reasons.push(format!("un autre livre de {}", candidate.authors));
        }
        if thematic_similarity >= 0.16 {
            score += 10.0;
            reasons.push(format!("proche de « {} »", nearest_book.title));
        }

        let nearest_tokens = book_tokens(nearest_book);
        let mut overlap = nearest_tokens
            .intersection(&candidate_tokens)
            .filter(|token| token.len() >= 5)
            .take(3)
            .cloned()
            .collect::<Vec<_>>();
        overlap.sort();
        if overlap.len() >= 2 {
            reasons.push(format!("points communs : {}", overlap.join(", ")));
        }
        if mood_similarity > 0.08 && !mood.trim().is_empty() {
            reasons.push("correspond à votre recherche actuelle".to_string());
        }
        if let (Some(pages), Some(limit)) = (candidate.pages, max_pages) {
            if pages <= limit {
                score += 5.0;
                reasons.push(format!("{pages} pages"));
            } else {
                score -= ((pages - limit) as f64 / 12.0).min(20.0);
                warnings.push(format!("plus long que souhaité — {pages} pages"));
            }
        }
        if candidate.cover_url.is_empty() {
            score -= 3.0;
        } else {
            score += 2.0;
        }
        if candidate.subjects.is_empty() && candidate.description.is_empty() && !same_author {
            score -= 14.0;
            warnings.push("peu de métadonnées fiables".to_string());
        }
        if reasons.is_empty() {
            continue;
        }

        score = score.clamp(0.0, 96.0).round();
        if score < 38.0 {
            continue;
        }

        scored.push(Recommendation {
            book: candidate,
            score,
            reasons: reasons.into_iter().take(3).collect(),
            warnings: warnings.into_iter().take(1).collect(),
        });
    }

    scored.sort_by(|first, second| second.score.total_cmp(&first.score));

    let mut deduplicated = Vec::new();
    let mut seen_works = HashSet::new();
    for item in scored {
        let work_key = series_work_identity(&item.book.title)
            .or_else(|| (!item.book.work_key.is_empty()).then_some(item.book.work_key.clone()))
            .unwrap_or_else(|| {
                format!(
                    "{}::{}",
                    canonical_title(&item.book.title, &item.book.authors),
                    author_key(&item.book.authors)
                )
            });
        if work_key.starts_with("::") || !seen_works.insert(work_key) {
            continue;
        }
        deduplicated.push(item);
    }

    // Premier passage : un seul livre par auteur. Les passages suivants ne
    // complètent la sélection qu’en cas de besoin. La première ligne ne peut
    // donc plus être remplie uniquement par Rowling ou Oda.
    let mut recommendations = Vec::new();
    let mut selected_indexes = HashSet::new();
    let mut author_counts: HashMap<String, usize> = HashMap::new();
    for author_cap in 1..=3 {
        for (index, item) in deduplicated.iter().enumerate() {
            if selected_indexes.contains(&index) {
                continue;
            }
            let author = author_key(&item.book.authors);
            let current = *author_counts.get(&author).unwrap_or(&0);
            if current >= author_cap {
                continue;
            }
            selected_indexes.insert(index);
            *author_counts.entry(author).or_insert(0) += 1;
            recommendations.push(item.clone());
            if recommendations.len() >= output_limit {
                return Ok(recommendations);
            }
        }
    }

    Ok(recommendations)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonicalizes_catalog_titles() {
        assert_eq!(
            canonical_title("L'étranger Par Albert Camus", "Albert Camus"),
            "etranger"
        );
        assert_eq!(
            canonical_title("1984 / George Orwell ; traduit de l'anglais", "Orwell, George"),
            "1984"
        );
        assert_eq!(
            canonical_title(
                "Harry Potter, Tome 5, Harry Potter et l'ordre du Phénix",
                "J. K. Rowling"
            ),
            "harry potter ordre phenix"
        );
        assert_eq!(
            canonical_title(
                "Les Animaux fantastiques : le texte du film",
                "J. K. Rowling"
            ),
            "animaux fantastiques"
        );
    }

    #[test]
    fn detects_same_work_across_catalog_title_variants() {
        let book = Book {
            title: "L'étranger Par Albert Camus".to_string(),
            authors: "Albert Camus".to_string(),
            ..Book::default()
        };
        let candidate = SearchResult {
            title: "L'Étranger".to_string(),
            authors: "Albert Camus".to_string(),
            ..SearchResult::default()
        };
        assert!(same_work(&book, &candidate));
    }

    #[test]
    fn detects_owned_work_from_any_edition_isbn() {
        let book = Book {
            isbn: Some("9782070643066".to_string()),
            title: "Harry Potter et l'Ordre du Phénix".to_string(),
            authors: "J. K. Rowling".to_string(),
            ..Book::default()
        };
        let candidate = SearchResult {
            isbn: Some("9781408855690".to_string()),
            alternate_isbns: vec![
                "9781408855690".to_string(),
                "9782070643066".to_string(),
            ],
            title: "Harry Potter and the Order of the Phoenix".to_string(),
            authors: "J. K. Rowling".to_string(),
            ..SearchResult::default()
        };
        assert!(same_work(&book, &candidate));
    }

    #[test]
    fn detects_harry_potter_across_languages_without_isbn() {
        let book = Book {
            title: "Harry Potter et le Prince de Sang-Mêlé".to_string(),
            authors: "J. K. Rowling".to_string(),
            ..Book::default()
        };
        let candidate = SearchResult {
            title: "Harry Potter and the Half-Blood Prince".to_string(),
            authors: "J. K. Rowling".to_string(),
            ..SearchResult::default()
        };
        assert!(same_work(&book, &candidate));
    }

    #[test]
    fn detects_fantastic_beasts_with_edition_suffix() {
        let book = Book {
            title: "Les Animaux fantastiques : le texte du film".to_string(),
            authors: "J. K. Rowling".to_string(),
            ..Book::default()
        };
        let candidate = SearchResult {
            title: "Les Animaux fantastiques".to_string(),
            authors: "J. K. Rowling, Olivia Lomenech Gill".to_string(),
            ..SearchResult::default()
        };
        assert!(same_work(&book, &candidate));
    }

    #[test]
    fn chosen_genre_is_strict_and_does_not_turn_harry_potter_into_a_classic_or_history_book() {
        let candidate = SearchResult {
            title: "Harry Potter à l'école des sorciers".to_string(),
            authors: "J. K. Rowling".to_string(),
            subjects: "Fantasy fiction, Magic, Wizards".to_string(),
            ..SearchResult::default()
        };
        assert!(!genre_matches(&candidate, "classics"));
        assert!(!genre_matches(&candidate, "history"));
        assert!(!genre_matches(&candidate, "philosophy"));
        assert!(genre_matches(&candidate, "fantasy"));
    }

    #[test]
    fn recognizes_bnf_author_order_for_classics() {
        let candidate = SearchResult {
            title: "Pauline".to_string(),
            authors: "Dumas, Alexandre (1802-1870). Auteur du texte".to_string(),
            ..SearchResult::default()
        };
        assert!(genre_matches(&candidate, "classics"));
        assert!(genre_matches(&candidate, "history"));
    }

    #[test]
    fn a_single_generic_token_is_not_a_similarity() {
        let first = ["series".to_string(), "pirates".to_string()]
            .into_iter()
            .collect::<HashSet<_>>();
        let second = ["series".to_string(), "dragons".to_string()]
            .into_iter()
            .collect::<HashSet<_>>();
        assert_eq!(similarity(&first, &second), 0.0);
    }

    #[test]
    fn groups_library_editions_only_for_recommendations() {
        let first = Book {
            title: "Harry Potter et l'Ordre du Phénix".to_string(),
            authors: "J. K. Rowling".to_string(),
            isbn: Some("9782070643066".to_string()),
            ..Book::default()
        };
        let second = Book {
            title: "Harry Potter, Tome 5, Harry Potter et l'ordre du Phénix".to_string(),
            authors: "J.K. Rowling".to_string(),
            isbn: Some("9782070543519".to_string()),
            ..Book::default()
        };
        assert!(same_library_work(&first, &second));
    }
}
