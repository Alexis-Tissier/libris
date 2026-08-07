use crate::models::SearchResult;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use reqwest::Client;
use serde::Deserialize;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::time::Duration;

const GOOGLE_URL: &str = "https://www.googleapis.com/books/v1/volumes";
const OPEN_LIBRARY_URL: &str = "https://openlibrary.org/search.json";
const BNF_URL: &str = "https://catalogue.bnf.fr/api/SRU";
const USER_AGENT: &str = "Libris/0.5.0 (desktop app)";

fn client() -> Result<Client, String> {
    Client::builder()
        .user_agent(USER_AGENT)
        .timeout(Duration::from_secs(14))
        .connect_timeout(Duration::from_secs(5))
        .build()
        .map_err(|error| format!("Impossible de préparer la recherche bibliographique : {error}"))
}

pub fn normalize_isbn(value: &str) -> String {
    value
        .chars()
        .filter(|character| character.is_ascii_digit() || *character == 'X' || *character == 'x')
        .map(|character| character.to_ascii_uppercase())
        .collect()
}

fn normalize_text(value: &str) -> String {
    value
        .to_lowercase()
        .chars()
        .map(|character| match character {
            'à' | 'á' | 'â' | 'ä' | 'ã' | 'å' => 'a',
            'ç' => 'c',
            'è' | 'é' | 'ê' | 'ë' => 'e',
            'ì' | 'í' | 'î' | 'ï' => 'i',
            'ñ' => 'n',
            'ò' | 'ó' | 'ô' | 'ö' | 'õ' => 'o',
            'ù' | 'ú' | 'û' | 'ü' => 'u',
            'ý' | 'ÿ' => 'y',
            character if character.is_alphanumeric() => character,
            _ => ' ',
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn tokens(value: &str) -> Vec<String> {
    const STOPWORDS: &[&str] = &[
        "de", "du", "des", "la", "le", "les", "un", "une", "et", "a", "au", "aux", "d", "l",
    ];
    normalize_text(value)
        .split_whitespace()
        .filter(|token| token.len() > 1 && !STOPWORDS.contains(token))
        .map(ToOwned::to_owned)
        .collect()
}

fn year_from(value: &str) -> Option<i64> {
    value
        .as_bytes()
        .windows(4)
        .filter_map(|window| std::str::from_utf8(window).ok())
        .find_map(|candidate| {
            if candidate.chars().all(|character| character.is_ascii_digit()) {
                candidate.parse::<i64>().ok().filter(|year| (1000..=2200).contains(year))
            } else {
                None
            }
        })
}

fn pages_from(value: &str) -> Option<i64> {
    let normalized = normalize_text(value);
    let words = normalized.split_whitespace().collect::<Vec<_>>();
    for (index, word) in words.iter().enumerate() {
        if (*word == "p" || *word == "pages" || *word == "page") && index > 0 {
            if let Ok(pages) = words[index - 1].parse::<i64>() {
                if (1..=100_000).contains(&pages) {
                    return Some(pages);
                }
            }
        }
    }
    None
}

fn extract_collection(values: &[String]) -> String {
    for value in values {
        let lower = value.to_lowercase();
        if let Some(index) = lower.find("collection") {
            let tail = value[index + "collection".len()..]
                .trim_matches(|character: char| character == ':' || character == ';' || character == '-' || character.is_whitespace());
            if !tail.is_empty() {
                return tail.split([';', '.']).next().unwrap_or(tail).trim().to_string();
            }
        }
        for marker in ["classico lycée", "classicolycée", "classico college", "classicocollège"] {
            if lower.contains(marker) {
                return value.trim().to_string();
            }
        }
    }
    String::new()
}

fn best_isbn(values: &[String]) -> Option<String> {
    values
        .iter()
        .map(|value| normalize_isbn(value))
        .find(|value| value.len() == 13)
        .or_else(|| {
            values
                .iter()
                .map(|value| normalize_isbn(value))
                .find(|value| value.len() == 10)
        })
}

fn score_result(result: &mut SearchResult, query: &str) {
    let query_isbn = normalize_isbn(query);
    let query_tokens = tokens(query);
    let haystack = normalize_text(&format!(
        "{} {} {} {} {} {} {} {}",
        result.title,
        result.authors,
        result.publisher,
        result.collection,
        result.published_date,
        result.description,
        result.subjects,
        result.isbn.clone().unwrap_or_default()
    ));

    let mut score = 0.0;
    let mut reasons = Vec::new();

    if matches!(query_isbn.len(), 10 | 13)
        && result
            .isbn
            .as_ref()
            .is_some_and(|isbn| normalize_isbn(isbn) == query_isbn)
    {
        score += 1_000.0;
        reasons.push("ISBN exact".to_string());
    }

    if !query_tokens.is_empty() {
        let matched = query_tokens
            .iter()
            .filter(|token| haystack.split_whitespace().any(|word| word == token.as_str()) || haystack.contains(token.as_str()))
            .count();
        let coverage = matched as f64 / query_tokens.len() as f64;
        score += coverage * 150.0;
        if matched == query_tokens.len() {
            score += 55.0;
            reasons.push("tous les termes correspondent".to_string());
        } else if coverage >= 0.7 {
            reasons.push("correspondance très proche".to_string());
        }
    }

    let normalized_query = normalize_text(query);
    let normalized_title = normalize_text(&result.title);
    if !normalized_title.is_empty() && normalized_query.contains(&normalized_title) {
        score += 35.0;
        reasons.push("titre exact".to_string());
    }
    if !result.publisher.is_empty() && normalized_query.contains(&normalize_text(&result.publisher)) {
        score += 30.0;
        reasons.push(format!("éditeur : {}", result.publisher));
    }
    if !result.collection.is_empty() && normalized_query.contains(&normalize_text(&result.collection)) {
        score += 35.0;
        reasons.push(format!("collection : {}", result.collection));
    }
    if result.language == "fr" || result.language == "fre" || result.language == "fra" {
        score += 10.0;
    }
    if result.source == "BnF" {
        score += 8.0;
    }
    if !result.cover_url.is_empty() {
        score += 4.0;
    }
    if result.pages.is_some() {
        score += 3.0;
    }
    if result.isbn.is_some() {
        score += 8.0;
    }

    result.relevance_score = score;
    result.match_reasons = reasons.into_iter().take(3).collect();
}

fn merge_result(target: &mut SearchResult, candidate: SearchResult) {
    if candidate.relevance_score > target.relevance_score {
        let mut better = candidate;
        if better.description.is_empty() {
            better.description = target.description.clone();
        }
        if better.subjects.is_empty() {
            better.subjects = target.subjects.clone();
        }
        if better.collection.is_empty() {
            better.collection = target.collection.clone();
        }
        if better.publisher.is_empty() {
            better.publisher = target.publisher.clone();
        }
        if better.cover_url.is_empty() {
            better.cover_url = target.cover_url.clone();
        }
        *target = better;
        return;
    }

    if target.description.is_empty() {
        target.description = candidate.description;
    }
    if target.subjects.is_empty() {
        target.subjects = candidate.subjects;
    }
    if target.collection.is_empty() {
        target.collection = candidate.collection;
    }
    if target.publisher.is_empty() {
        target.publisher = candidate.publisher;
    }
    if target.cover_url.is_empty() {
        target.cover_url = candidate.cover_url;
    }
    if target.pages.is_none() {
        target.pages = candidate.pages;
    }
}

pub async fn search(query: &str, limit: usize) -> Result<Vec<SearchResult>, String> {
    let query = query.trim();
    if query.is_empty() {
        return Ok(Vec::new());
    }

    let client = client()?;
    let (google, open_library, bnf) = tokio::join!(
        search_google(&client, query),
        search_open_library_editions(&client, query),
        search_bnf(&client, query),
    );

    let mut errors = Vec::new();
    let mut all = Vec::new();
    match google {
        Ok(mut values) => all.append(&mut values),
        Err(error) => errors.push(format!("Google Books : {error}")),
    }
    match open_library {
        Ok(mut values) => all.append(&mut values),
        Err(error) => errors.push(format!("Open Library : {error}")),
    }
    match bnf {
        Ok(mut values) => all.append(&mut values),
        Err(error) => errors.push(format!("BnF : {error}")),
    }

    if all.is_empty() && errors.len() == 3 {
        return Err(format!(
            "Les catalogues n’ont pas répondu. {}",
            errors.join(" • ")
        ));
    }

    for result in &mut all {
        score_result(result, query);
    }

    let mut merged: HashMap<String, SearchResult> = HashMap::new();
    for result in all {
        let key = result
            .isbn
            .as_ref()
            .map(|isbn| format!("isbn:{}", normalize_isbn(isbn)))
            .filter(|value| value.len() > 5)
            .unwrap_or_else(|| {
                format!(
                    "{}:{}:{}:{}:{}",
                    result.source,
                    result.source_id,
                    normalize_text(&result.title),
                    normalize_text(&result.publisher),
                    result.publish_year.unwrap_or_default()
                )
            });
        if let Some(existing) = merged.get_mut(&key) {
            merge_result(existing, result);
        } else {
            merged.insert(key, result);
        }
    }

    let mut results = merged.into_values().collect::<Vec<_>>();
    results.sort_by(|left, right| {
        right
            .relevance_score
            .total_cmp(&left.relevance_score)
            .then_with(|| left.title.cmp(&right.title))
    });
    results.truncate(limit.min(80));
    Ok(results)
}

#[derive(Debug, Deserialize)]
struct GoogleResponse {
    #[serde(default)]
    items: Vec<GoogleVolume>,
}

#[derive(Debug, Deserialize)]
struct GoogleVolume {
    #[serde(default)]
    id: String,
    #[serde(rename = "volumeInfo", default)]
    volume_info: GoogleVolumeInfo,
}

#[derive(Debug, Default, Deserialize)]
struct GoogleVolumeInfo {
    #[serde(default)]
    title: String,
    #[serde(default)]
    subtitle: String,
    #[serde(default)]
    authors: Vec<String>,
    #[serde(default)]
    publisher: String,
    #[serde(rename = "publishedDate", default)]
    published_date: String,
    #[serde(default)]
    description: String,
    #[serde(rename = "industryIdentifiers", default)]
    industry_identifiers: Vec<GoogleIdentifier>,
    #[serde(rename = "pageCount")]
    page_count: Option<i64>,
    #[serde(default)]
    categories: Vec<String>,
    #[serde(default)]
    language: String,
    #[serde(rename = "imageLinks", default)]
    image_links: GoogleImageLinks,
    #[serde(rename = "printType", default)]
    print_type: String,
    #[serde(rename = "seriesInfo")]
    series_info: Option<GoogleSeriesInfo>,
}

#[derive(Debug, Deserialize)]
struct GoogleIdentifier {
    #[serde(rename = "type", default)]
    kind: String,
    #[serde(default)]
    identifier: String,
}

#[derive(Debug, Default, Deserialize)]
struct GoogleImageLinks {
    #[serde(default)]
    thumbnail: String,
    #[serde(default)]
    small_thumbnail: String,
    #[serde(default)]
    medium: String,
    #[serde(default)]
    large: String,
}

#[derive(Debug, Deserialize)]
struct GoogleSeriesInfo {
    #[serde(rename = "bookDisplayNumber", default)]
    book_display_number: String,
}

async fn search_google(client: &Client, query: &str) -> Result<Vec<SearchResult>, String> {
    let normalized_isbn = normalize_isbn(query);
    let effective_query = if matches!(normalized_isbn.len(), 10 | 13) {
        format!("isbn:{normalized_isbn}")
    } else {
        query.to_string()
    };

    let response = client
        .get(GOOGLE_URL)
        .query(&[
            ("q", effective_query.as_str()),
            ("maxResults", "40"),
            ("printType", "books"),
            ("orderBy", "relevance"),
            ("langRestrict", "fr"),
            ("projection", "full"),
        ])
        .send()
        .await
        .map_err(|error| network_error("Google Books", error))?;

    if !response.status().is_success() {
        return Err(format!("erreur HTTP {}", response.status().as_u16()));
    }

    let payload = response
        .json::<GoogleResponse>()
        .await
        .map_err(|error| format!("réponse invalide : {error}"))?;

    Ok(payload
        .items
        .into_iter()
        .filter_map(|volume| {
            let info = volume.volume_info;
            if info.title.trim().is_empty() {
                return None;
            }
            let identifiers = info
                .industry_identifiers
                .iter()
                .filter(|identifier| identifier.kind.starts_with("ISBN"))
                .map(|identifier| identifier.identifier.clone())
                .collect::<Vec<_>>();
            let isbn = best_isbn(&identifiers);
            let cover = [
                info.image_links.large,
                info.image_links.medium,
                info.image_links.thumbnail,
                info.image_links.small_thumbnail,
            ]
            .into_iter()
            .find(|value| !value.is_empty())
            .unwrap_or_default()
            .replace("http://", "https://");
            let collection = info
                .series_info
                .as_ref()
                .map(|series| series.book_display_number.clone())
                .filter(|value| !value.is_empty())
                .unwrap_or_else(|| {
                    let subtitle_lower = info.subtitle.to_lowercase();
                    if subtitle_lower.contains("collection") || subtitle_lower.contains("classico") {
                        info.subtitle.clone()
                    } else {
                        String::new()
                    }
                });

            Some(SearchResult {
                work_key: format!("google:{}", volume.id),
                edition_key: None,
                alternate_isbns: isbn.clone().into_iter().collect(),
                isbn,
                title: info.title.trim().to_string(),
                authors: if info.authors.is_empty() {
                    "Auteur inconnu".to_string()
                } else {
                    info.authors.join(", ")
                },
                description: info.description,
                subjects: info.categories.join(", "),
                publish_year: year_from(&info.published_date),
                published_date: info.published_date,
                pages: info.page_count,
                language: if info.language.is_empty() { "fr".to_string() } else { info.language },
                cover_url: cover,
                edition_count: 1,
                publisher: info.publisher,
                collection,
                format: info.print_type,
                source: "Google Books".to_string(),
                source_id: volume.id,
                relevance_score: 0.0,
                match_reasons: Vec::new(),
            })
        })
        .collect())
}

fn value_strings(value: Option<&Value>) -> Vec<String> {
    match value {
        Some(Value::String(text)) => vec![text.clone()],
        Some(Value::Array(values)) => values
            .iter()
            .filter_map(|value| match value {
                Value::String(text) => Some(text.clone()),
                Value::Object(map) => map
                    .get("value")
                    .and_then(Value::as_str)
                    .or_else(|| map.get("key").and_then(Value::as_str))
                    .or_else(|| map.get("name").and_then(Value::as_str))
                    .map(ToOwned::to_owned),
                _ => None,
            })
            .collect(),
        Some(Value::Object(map)) => map
            .get("value")
            .and_then(Value::as_str)
            .or_else(|| map.get("name").and_then(Value::as_str))
            .map(|value| vec![value.to_string()])
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

fn open_library_cover(edition: &Value, isbn: Option<&String>, edition_key: &str) -> String {
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

async fn search_open_library_editions(client: &Client, query: &str) -> Result<Vec<SearchResult>, String> {
    let normalized_isbn = normalize_isbn(query);
    let effective_query = if matches!(normalized_isbn.len(), 10 | 13) {
        format!("isbn:{normalized_isbn}")
    } else {
        query.to_string()
    };

    let response = client
        .get(OPEN_LIBRARY_URL)
        .query(&[
            ("q", effective_query.as_str()),
            ("limit", "30"),
            ("lang", "fr"),
            ("mode", "everything"),
            (
                "fields",
                "key,title,author_name,subject,description,first_sentence,edition_count,editions",
            ),
        ])
        .send()
        .await
        .map_err(|error| network_error("Open Library", error))?;

    if !response.status().is_success() {
        return Err(format!("erreur HTTP {}", response.status().as_u16()));
    }

    let payload = response
        .json::<Value>()
        .await
        .map_err(|error| format!("réponse invalide : {error}"))?;
    let docs = payload
        .get("docs")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();

    let mut results = Vec::new();
    for doc in docs {
        let work_key = value_string(doc.get("key"));
        let work_title = value_string(doc.get("title"));
        let authors = value_strings(doc.get("author_name"));
        let subjects = value_strings(doc.get("subject"));
        let description = value_string(doc.get("description"));
        let first_sentence = value_string(doc.get("first_sentence"));
        let edition_count = value_i64(doc.get("edition_count")).unwrap_or_default();
        let edition_docs = doc
            .get("editions")
            .and_then(|value| value.get("docs"))
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();

        if edition_docs.is_empty() {
            continue;
        }

        for edition in edition_docs.into_iter().take(20) {
            let title = value_string(edition.get("title"));
            let title = if title.is_empty() { work_title.clone() } else { title };
            if title.is_empty() {
                continue;
            }
            let publishers = {
                let mut values = value_strings(edition.get("publisher"));
                values.extend(value_strings(edition.get("publishers")));
                values.sort();
                values.dedup();
                values
            };
            let isbn_values = {
                let mut values = value_strings(edition.get("isbn"));
                values.extend(value_strings(edition.get("isbn_13")));
                values.extend(value_strings(edition.get("isbn_10")));
                values
            };
            let isbn = best_isbn(&isbn_values);
            let series = value_strings(edition.get("series"));
            let publish_date = value_string(edition.get("publish_date"));
            let edition_key = value_string(edition.get("key"));
            let language_values = {
                let mut values = value_strings(edition.get("language"));
                values.extend(value_strings(edition.get("languages")));
                values
            };
            let language = language_values
                .iter()
                .find(|language| language.contains("fre") || language.as_str() == "fr")
                .cloned()
                .or_else(|| language_values.first().cloned())
                .unwrap_or_else(|| "fr".to_string())
                .trim_start_matches("/languages/")
                .to_string();

            results.push(SearchResult {
                work_key: work_key.clone(),
                edition_key: (!edition_key.is_empty()).then_some(edition_key.clone()),
                alternate_isbns: isbn.clone().into_iter().collect(),
                isbn: isbn.clone(),
                title,
                authors: if authors.is_empty() {
                    "Auteur inconnu".to_string()
                } else {
                    authors.join(", ")
                },
                description: if description.is_empty() { first_sentence.clone() } else { description.clone() },
                subjects: subjects.iter().take(24).cloned().collect::<Vec<_>>().join(", "),
                publish_year: value_i64(edition.get("publish_year")).or_else(|| year_from(&publish_date)),
                published_date: publish_date,
                pages: value_i64(edition.get("number_of_pages"))
                    .or_else(|| value_i64(edition.get("number_of_pages_median"))),
                language,
                cover_url: open_library_cover(&edition, isbn.as_ref(), &edition_key),
                edition_count,
                publisher: publishers.join(", "),
                collection: series.join(", "),
                format: value_string(edition.get("physical_format")),
                source: "Open Library".to_string(),
                source_id: edition_key,
                relevance_score: 0.0,
                match_reasons: Vec::new(),
            });
        }
    }

    Ok(results)
}

fn xml_decode(value: &str) -> String {
    let mut output = value
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'");

    loop {
        let Some(start) = output.find("&#") else { break };
        let Some(relative_end) = output[start..].find(';') else { break };
        let end = start + relative_end;
        let entity = &output[start + 2..end];
        let parsed = if let Some(hex) = entity.strip_prefix('x').or_else(|| entity.strip_prefix('X')) {
            u32::from_str_radix(hex, 16).ok()
        } else {
            entity.parse::<u32>().ok()
        };
        let Some(character) = parsed.and_then(char::from_u32) else { break };
        output.replace_range(start..=end, &character.to_string());
    }

    output.trim().to_string()
}

fn strip_xml_tags(value: &str) -> String {
    let mut output = String::new();
    let mut in_tag = false;
    for character in value.chars() {
        match character {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => output.push(character),
            _ => {}
        }
    }
    xml_decode(&output)
}

fn extract_xml_values(xml: &str, tag: &str) -> Vec<String> {
    let mut values = Vec::new();
    let opening = format!("<{tag}");
    let closing = format!("</{tag}>");
    let mut offset = 0;
    while let Some(relative_start) = xml[offset..].find(&opening) {
        let start = offset + relative_start;
        let Some(relative_gt) = xml[start..].find('>') else { break };
        let content_start = start + relative_gt + 1;
        let Some(relative_end) = xml[content_start..].find(&closing) else { break };
        let end = content_start + relative_end;
        let value = strip_xml_tags(&xml[content_start..end]);
        if !value.is_empty() {
            values.push(value);
        }
        offset = end + closing.len();
    }
    values
}

fn bnf_isbn(identifiers: &[String]) -> Option<String> {
    identifiers
        .iter()
        .filter(|identifier| identifier.to_lowercase().contains("isbn"))
        .map(|identifier| normalize_isbn(identifier))
        .find(|isbn| matches!(isbn.len(), 10 | 13))
        .or_else(|| best_isbn(identifiers))
}

async fn search_bnf(client: &Client, query: &str) -> Result<Vec<SearchResult>, String> {
    let normalized_isbn = normalize_isbn(query);
    let cql = if matches!(normalized_isbn.len(), 10 | 13) {
        format!("bib.isbn all \"{normalized_isbn}\"")
    } else {
        let escaped = query.replace('"', " ");
        format!("(bib.anywhere all \"{escaped}\") and (bib.recordtype any \"mon\")")
    };

    let response = client
        .get(BNF_URL)
        .query(&[
            ("version", "1.2"),
            ("operation", "searchRetrieve"),
            ("query", cql.as_str()),
            ("recordSchema", "dublincore"),
            ("maximumRecords", "40"),
        ])
        .send()
        .await
        .map_err(|error| network_error("BnF", error))?;

    if !response.status().is_success() {
        return Err(format!("erreur HTTP {}", response.status().as_u16()));
    }

    let xml = response
        .text()
        .await
        .map_err(|error| format!("réponse illisible : {error}"))?;
    let mut results = Vec::new();

    for chunk in xml.split("<srw:recordData>").skip(1) {
        let record = chunk.split("</srw:recordData>").next().unwrap_or(chunk);
        let titles = extract_xml_values(record, "dc:title");
        let title = titles.first().cloned().unwrap_or_default();
        if title.is_empty() {
            continue;
        }
        let creators = extract_xml_values(record, "dc:creator");
        let publishers = extract_xml_values(record, "dc:publisher");
        let mut descriptions = extract_xml_values(record, "dc:description");
        descriptions.extend(extract_xml_values(record, "dc:relation"));
        let subjects = extract_xml_values(record, "dc:subject");
        let identifiers = extract_xml_values(record, "dc:identifier");
        let dates = extract_xml_values(record, "dc:date");
        let formats = extract_xml_values(record, "dc:format");
        let languages = extract_xml_values(record, "dc:language");
        let isbn = bnf_isbn(&identifiers);
        let ark = identifiers
            .iter()
            .find_map(|identifier| identifier.find("ark:/12148/").map(|index| identifier[index..].to_string()))
            .unwrap_or_default();
        let description_text = descriptions.join(" • ");
        let collection = extract_collection(&descriptions);
        let pages = descriptions
            .iter()
            .chain(formats.iter())
            .find_map(|value| pages_from(value));
        let published_date = dates.first().cloned().unwrap_or_default();
        let cover_url = if let Some(isbn) = isbn.as_ref() {
            let parameter = if normalize_isbn(isbn).len() == 13 { "EAN" } else { "ISBN" };
            format!(
                "https://openapi.bnf.fr/couverture/image/image/recupererImage?{parameter}={}&couverture=1&taille=originale&largeur=500&hauteur=700",
                urlencoding::encode(isbn)
            )
        } else if !ark.is_empty() {
            format!(
                "https://openapi.bnf.fr/couverture/image/image/recupererImage?idArk={}&couverture=1&taille=originale&largeur=500&hauteur=700",
                urlencoding::encode(&ark)
            )
        } else {
            String::new()
        };

        results.push(SearchResult {
            work_key: if ark.is_empty() { format!("bnf:{title}") } else { ark.clone() },
            edition_key: None,
            alternate_isbns: isbn.clone().into_iter().collect(),
            isbn,
            title,
            authors: if creators.is_empty() { "Auteur inconnu".to_string() } else { creators.join(", ") },
            description: description_text,
            subjects: subjects.into_iter().take(24).collect::<Vec<_>>().join(", "),
            publish_year: year_from(&published_date),
            published_date,
            pages,
            language: languages.first().cloned().unwrap_or_else(|| "fr".to_string()),
            cover_url,
            edition_count: 1,
            publisher: publishers.join(", "),
            collection,
            format: formats.join(", "),
            source: "BnF".to_string(),
            source_id: ark,
            relevance_score: 0.0,
            match_reasons: Vec::new(),
        });
    }

    Ok(results)
}

fn image_mime(bytes: &[u8], content_type: Option<&str>) -> Option<&'static str> {
    if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
        return Some("image/jpeg");
    }
    if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        return Some("image/png");
    }
    if bytes.len() >= 12 && &bytes[..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
        return Some("image/webp");
    }
    if bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a") {
        return Some("image/gif");
    }
    match content_type.unwrap_or_default().split(';').next().unwrap_or_default().trim() {
        "image/jpeg" | "image/jpg" => Some("image/jpeg"),
        "image/png" => Some("image/png"),
        "image/webp" => Some("image/webp"),
        "image/gif" => Some("image/gif"),
        _ => None,
    }
}

async fn image_as_data_url(client: &Client, url: &str) -> Option<String> {
    if url.trim().is_empty() {
        return None;
    }
    let response = client
        .get(url)
        .header("Accept", "image/avif,image/webp,image/apng,image/svg+xml,image/*,*/*;q=0.8")
        .send()
        .await
        .ok()?;
    if !response.status().is_success() {
        return None;
    }
    let content_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned);
    let bytes = response.bytes().await.ok()?;
    if bytes.len() < 900 || bytes.len() > 8 * 1024 * 1024 {
        return None;
    }
    let mime = image_mime(&bytes, content_type.as_deref())?;
    Some(format!("data:{mime};base64,{}", BASE64.encode(bytes.as_ref())))
}

async fn google_volume_cover(client: &Client, source_id: &str) -> Option<String> {
    if source_id.trim().is_empty() {
        return None;
    }
    let url = format!("{GOOGLE_URL}/{}", urlencoding::encode(source_id));
    let payload = client.get(url).send().await.ok()?.json::<GoogleVolume>().await.ok()?;
    let info = payload.volume_info;
    [
        info.image_links.large,
        info.image_links.medium,
        info.image_links.thumbnail,
        info.image_links.small_thumbnail,
    ]
    .into_iter()
    .find(|value| !value.is_empty())
    .map(|value| value.replace("http://", "https://"))
}

pub async fn resolve_cover(
    cover_url: &str,
    isbn: Option<&str>,
    source: &str,
    source_id: &str,
) -> Result<Option<String>, String> {
    let client = client()?;
    let mut candidates = Vec::new();
    let mut seen = HashSet::new();

    fn push_candidate(candidates: &mut Vec<String>, seen: &mut HashSet<String>, value: String) {
        let value = value.trim().to_string();
        if !value.is_empty() && seen.insert(value.clone()) {
            candidates.push(value);
        }
    }

    push_candidate(
        &mut candidates,
        &mut seen,
        cover_url.replace("http://", "https://"),
    );

    let normalized_isbn = isbn.map(normalize_isbn).unwrap_or_default();
    if matches!(normalized_isbn.len(), 10 | 13) {
        push_candidate(
            &mut candidates,
            &mut seen,
            format!("https://covers.openlibrary.org/b/isbn/{normalized_isbn}-L.jpg?default=false"),
        );
        let parameter = if normalized_isbn.len() == 13 { "EAN" } else { "ISBN" };
        push_candidate(
            &mut candidates,
            &mut seen,
            format!(
                "https://openapi.bnf.fr/couverture/image/image/recupererImage?{parameter}={}&couverture=1&taille=originale&largeur=700&hauteur=1000",
                urlencoding::encode(&normalized_isbn)
            ),
        );
    }

    if source == "Open Library" {
        let olid = source_id.trim_start_matches("/books/");
        if !olid.is_empty() {
            push_candidate(
                &mut candidates,
                &mut seen,
                format!("https://covers.openlibrary.org/b/olid/{olid}-L.jpg?default=false"),
            );
        }
    } else if source == "BnF" && !source_id.trim().is_empty() {
        push_candidate(
            &mut candidates,
            &mut seen,
            format!(
                "https://openapi.bnf.fr/couverture/image/image/recupererImage?idArk={}&couverture=1&taille=originale&largeur=700&hauteur=1000",
                urlencoding::encode(source_id)
            ),
        );
    }

    for candidate in candidates {
        if let Some(data_url) = image_as_data_url(&client, &candidate).await {
            return Ok(Some(data_url));
        }
    }

    // Dernier recours fiable : Google Books interrogé par ISBN exact.
    if matches!(normalized_isbn.len(), 10 | 13) {
        if let Ok(results) = search_google(&client, &normalized_isbn).await {
            for result in results {
                let exact = result
                    .isbn
                    .as_deref()
                    .is_some_and(|value| normalize_isbn(value) == normalized_isbn);
                if exact && !result.cover_url.is_empty() {
                    if let Some(data_url) = image_as_data_url(&client, &result.cover_url).await {
                        return Ok(Some(data_url));
                    }
                }
            }
        }
    }

    if source == "Google Books" && cover_url.trim().is_empty() {
        if let Some(url) = google_volume_cover(&client, source_id).await {
            if let Some(data_url) = image_as_data_url(&client, &url).await {
                return Ok(Some(data_url));
            }
        }
    }

    Ok(None)
}

fn network_error(source: &str, error: reqwest::Error) -> String {
    if error.is_timeout() {
        format!("{source} met trop de temps à répondre")
    } else if error.is_connect() {
        format!("connexion à {source} impossible")
    } else {
        error.to_string()
    }
}

pub async fn enrich(result: SearchResult) -> SearchResult {
    if result.source == "Open Library" && result.work_key.starts_with("/works/") {
        return crate::openlibrary::enrich(result).await;
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn isbn_normalization() {
        assert_eq!(normalize_isbn("978-2-7011-9291-8"), "9782701192918");
    }

    #[test]
    fn french_edition_keywords_rank_high() {
        let mut result = SearchResult {
            title: "Pauline".to_string(),
            authors: "Alexandre Dumas".to_string(),
            publisher: "Belin Gallimard".to_string(),
            collection: "Classico Lycée".to_string(),
            isbn: Some("9782701192918".to_string()),
            language: "fr".to_string(),
            source: "BnF".to_string(),
            ..SearchResult::default()
        };
        score_result(&mut result, "Pauline Alexandre Dumas Belin Gallimard Classico Lycée");
        assert!(result.relevance_score > 200.0);
        assert!(result.match_reasons.iter().any(|reason| reason.contains("termes")));
    }

    #[test]
    fn parse_pages_from_bnf_description() {
        assert_eq!(pages_from("1 vol. (224 p.)"), Some(224));
    }
}
