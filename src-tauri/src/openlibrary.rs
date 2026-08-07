use crate::models::SearchResult;
use reqwest::Client;
use serde_json::Value;
use std::collections::HashSet;
use std::time::Duration;

const SEARCH_URL: &str = "https://openlibrary.org/search.json";
const USER_AGENT: &str = "Libris/0.5.0 (desktop app)";

fn client() -> Result<Client, String> {
    Client::builder()
        .user_agent(USER_AGENT)
        .timeout(Duration::from_secs(14))
        .connect_timeout(Duration::from_secs(5))
        .build()
        .map_err(|error| format!("Impossible de préparer la connexion à Open Library : {error}"))
}

fn normalize_isbn(value: &str) -> String {
    value
        .chars()
        .filter(|character| character.is_ascii_digit() || *character == 'X' || *character == 'x')
        .map(|character| character.to_ascii_uppercase())
        .collect()
}

fn value_strings(value: Option<&Value>) -> Vec<String> {
    match value {
        Some(Value::String(text)) => vec![text.trim().to_string()],
        Some(Value::Array(values)) => values
            .iter()
            .filter_map(|value| match value {
                Value::String(text) => Some(text.trim().to_string()),
                Value::Object(map) => map
                    .get("value")
                    .and_then(Value::as_str)
                    .or_else(|| map.get("name").and_then(Value::as_str))
                    .or_else(|| map.get("key").and_then(Value::as_str))
                    .map(|text| text.trim().to_string()),
                _ => None,
            })
            .filter(|value| !value.is_empty())
            .collect(),
        Some(Value::Object(map)) => map
            .get("value")
            .and_then(Value::as_str)
            .or_else(|| map.get("name").and_then(Value::as_str))
            .or_else(|| map.get("key").and_then(Value::as_str))
            .map(|value| vec![value.trim().to_string()])
            .unwrap_or_default(),
        _ => Vec::new(),
    }
}

fn value_string(value: Option<&Value>) -> String {
    value_strings(value).into_iter().next().unwrap_or_default()
}

fn value_i64(value: Option<&Value>) -> Option<i64> {
    value.and_then(|value| {
        value
            .as_i64()
            .or_else(|| value.as_str().and_then(|text| text.parse::<i64>().ok()))
    })
}

fn year_from(value: &str) -> Option<i64> {
    value
        .as_bytes()
        .windows(4)
        .filter_map(|window| std::str::from_utf8(window).ok())
        .find_map(|candidate| {
            candidate
                .chars()
                .all(|character| character.is_ascii_digit())
                .then(|| candidate.parse::<i64>().ok())
                .flatten()
                .filter(|year| (1000..=2200).contains(year))
        })
}

fn language_values(edition: &Value) -> Vec<String> {
    let mut values = value_strings(edition.get("language"));
    values.extend(value_strings(edition.get("languages")));
    values
        .into_iter()
        .map(|value| value.trim_start_matches("/languages/").to_lowercase())
        .collect()
}

fn is_french_edition(edition: &Value) -> bool {
    language_values(edition)
        .iter()
        .any(|language| matches!(language.as_str(), "fr" | "fre" | "fra" | "french"))
}

fn edition_isbns(edition: &Value) -> Vec<String> {
    let mut values = value_strings(edition.get("isbn"));
    values.extend(value_strings(edition.get("isbn_13")));
    values.extend(value_strings(edition.get("isbn_10")));
    values
        .into_iter()
        .map(|value| normalize_isbn(&value))
        .filter(|value| matches!(value.len(), 10 | 13))
        .collect()
}

fn best_isbn(values: &[String]) -> Option<String> {
    values
        .iter()
        .find(|value| value.len() == 13)
        .cloned()
        .or_else(|| values.first().cloned())
}

fn cover_url(edition: &Value, isbn: Option<&str>, edition_key: &str) -> String {
    if let Some(id) = value_i64(edition.get("cover_i")) {
        return format!("https://covers.openlibrary.org/b/id/{id}-L.jpg?default=false");
    }
    if let Some(id) = edition
        .get("covers")
        .and_then(Value::as_array)
        .and_then(|values| values.first())
        .and_then(Value::as_i64)
    {
        return format!("https://covers.openlibrary.org/b/id/{id}-L.jpg?default=false");
    }
    if let Some(isbn) = isbn {
        return format!("https://covers.openlibrary.org/b/isbn/{isbn}-L.jpg?default=false");
    }
    let olid = edition_key.trim_start_matches("/books/");
    if !olid.is_empty() {
        return format!("https://covers.openlibrary.org/b/olid/{olid}-L.jpg?default=false");
    }
    String::new()
}


fn title_words(value: &str) -> Vec<String> {
    value
        .to_lowercase()
        .chars()
        .map(|character| if character.is_alphanumeric() { character } else { ' ' })
        .collect::<String>()
        .split_whitespace()
        .map(ToOwned::to_owned)
        .collect()
}

fn looks_clearly_english_title(value: &str) -> bool {
    let words = title_words(value);
    if words.is_empty() {
        return false;
    }
    let markers = ["the", "and", "where", "through", "into", "from", "with", "without", "ages", "them", "their", "island"];
    let marker_count = words
        .iter()
        .filter(|word| markers.contains(&word.as_str()))
        .count();
    words.first().is_some_and(|word| word == "the") || marker_count >= 2
}

fn french_title_bonus(value: &str) -> i64 {
    let words = title_words(value);
    let markers = ["le", "la", "les", "de", "des", "du", "et", "une", "un", "aux", "dans", "pour"];
    if words.iter().any(|word| markers.contains(&word.as_str())) {
        18
    } else {
        0
    }
}

fn french_edition_score(edition: &Value) -> i64 {
    if !is_french_edition(edition) {
        return i64::MIN;
    }
    let title = value_string(edition.get("title"));
    if title.is_empty() || looks_clearly_english_title(&title) {
        return i64::MIN;
    }
    let mut score = 100_i64 + french_title_bonus(&title);
    let isbns = edition_isbns(edition);
    if isbns.iter().any(|isbn| isbn.len() == 13) {
        score += 20;
    }
    score += 12;
    if !value_strings(edition.get("publisher")).is_empty()
        || !value_strings(edition.get("publishers")).is_empty()
    {
        score += 8;
    }
    if value_i64(edition.get("number_of_pages")).is_some() {
        score += 5;
    }
    if value_i64(edition.get("cover_i")).is_some()
        || edition.get("covers").and_then(Value::as_array).is_some_and(|values| !values.is_empty())
    {
        score += 5;
    }
    score
}

pub async fn search(query: &str, limit: usize) -> Result<Vec<SearchResult>, String> {
    let query = query.trim();
    if query.is_empty() {
        return Ok(Vec::new());
    }

    let isbn = normalize_isbn(query);
    let effective_query = if matches!(isbn.len(), 10 | 13) {
        format!("isbn:{isbn}")
    } else {
        query.to_string()
    };

    let limit_value = limit.min(50).to_string();
    let response = client()?
        .get(SEARCH_URL)
        .query(&[
            ("q", effective_query.as_str()),
            ("limit", limit_value.as_str()),
            ("mode", "everything"),
            ("lang", "fr"),
            (
                "fields",
                "key,title,author_name,first_publish_year,isbn,language,subject,description,first_sentence,edition_count,editions",
            ),
        ])
        .send()
        .await
        .map_err(|error| {
            if error.is_timeout() {
                "Open Library met trop de temps à répondre. Réessaie dans quelques secondes.".to_string()
            } else if error.is_connect() {
                "Connexion à Open Library impossible. Vérifie ta connexion Internet.".to_string()
            } else {
                format!("Recherche Open Library impossible : {error}")
            }
        })?;

    if !response.status().is_success() {
        return Err(format!(
            "Open Library a répondu avec l’erreur HTTP {}.",
            response.status().as_u16()
        ));
    }

    let payload: Value = response
        .json()
        .await
        .map_err(|error| format!("La réponse d’Open Library est invalide : {error}"))?;
    let docs = payload
        .get("docs")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();

    let mut seen_works = HashSet::new();
    let mut results = Vec::new();

    for doc in docs {
        let work_key = value_string(doc.get("key"));
        if work_key.is_empty() || !seen_works.insert(work_key.clone()) {
            continue;
        }

        let authors = value_strings(doc.get("author_name"));
        if authors.is_empty() {
            continue;
        }

        let edition_docs = doc
            .get("editions")
            .and_then(|value| value.get("docs"))
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();

        // La recommandation doit afficher une vraie édition française. Le titre
        // de l’œuvre Open Library peut être anglais même quand une traduction
        // française existe : on choisit donc la meilleure édition marquée FR.
        let Some(edition) = edition_docs
            .iter()
            .filter(|edition| french_edition_score(edition) > i64::MIN)
            .max_by_key(|edition| french_edition_score(edition))
        else {
            continue;
        };

        let mut alternate_isbns = value_strings(doc.get("isbn"))
            .into_iter()
            .map(|value| normalize_isbn(&value))
            .filter(|value| matches!(value.len(), 10 | 13))
            .collect::<Vec<_>>();
        for candidate_edition in &edition_docs {
            alternate_isbns.extend(edition_isbns(candidate_edition));
        }
        alternate_isbns.sort();
        alternate_isbns.dedup();

        let selected_isbns = edition_isbns(edition);
        let primary_isbn = best_isbn(&selected_isbns)
            .or_else(|| alternate_isbns.iter().find(|value| value.len() == 13).cloned())
            .or_else(|| alternate_isbns.first().cloned());
        let edition_key = value_string(edition.get("key"));
        let edition_title = value_string(edition.get("title"));
        let work_title = value_string(doc.get("title"));
        let title = if edition_title.is_empty() { work_title } else { edition_title };
        if title.is_empty() {
            continue;
        }

        let mut publishers = value_strings(edition.get("publisher"));
        publishers.extend(value_strings(edition.get("publishers")));
        publishers.sort();
        publishers.dedup();

        let publish_date = value_string(edition.get("publish_date"));
        let series = value_strings(edition.get("series"));
        let description = value_string(doc.get("description"));
        let first_sentence = value_string(doc.get("first_sentence"));
        let subjects = value_strings(doc.get("subject"));

        results.push(SearchResult {
            work_key: work_key.clone(),
            edition_key: (!edition_key.is_empty()).then_some(edition_key.clone()),
            source: "Open Library".to_string(),
            source_id: edition_key.clone(),
            isbn: primary_isbn.clone(),
            alternate_isbns,
            title,
            authors: authors.join(", "),
            publisher: publishers.join(", "),
            collection: series.join(", "),
            published_date: publish_date.clone(),
            format: value_string(edition.get("physical_format")),
            description: if description.is_empty() { first_sentence } else { description },
            subjects: subjects.into_iter().take(24).collect::<Vec<_>>().join(", "),
            publish_year: value_i64(edition.get("publish_year"))
                .or_else(|| year_from(&publish_date))
                .or_else(|| value_i64(doc.get("first_publish_year"))),
            pages: value_i64(edition.get("number_of_pages"))
                .or_else(|| value_i64(edition.get("number_of_pages_median"))),
            language: "fr".to_string(),
            cover_url: cover_url(edition, primary_isbn.as_deref(), &edition_key),
            edition_count: value_i64(doc.get("edition_count")).unwrap_or_default(),
            relevance_score: 0.0,
            match_reasons: Vec::new(),
        });
    }

    Ok(results)
}

pub async fn enrich(result: SearchResult) -> SearchResult {
    // La recherche de recommandations récupère déjà la notice de l’œuvre et
    // l’édition française choisie. Aucun second appel n’est nécessaire.
    result
}
