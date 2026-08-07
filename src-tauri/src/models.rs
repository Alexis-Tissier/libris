use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Book {
    pub id: Option<i64>,
    pub work_key: Option<String>,
    pub edition_key: Option<String>,
    pub source: String,
    pub source_id: String,
    pub isbn: Option<String>,
    pub title: String,
    pub authors: String,
    pub publisher: String,
    pub collection: String,
    pub published_date: String,
    pub format: String,
    pub description: String,
    pub subjects: String,
    pub publish_year: Option<i64>,
    pub pages: Option<i64>,
    pub language: String,
    pub cover_url: String,
    pub cover_path: String,
    pub status: String,
    pub owned: bool,
    pub rating: Option<f64>,
    pub review: String,
    pub liked: String,
    pub disliked: String,
    pub location: String,
    pub purchase_price: Option<f64>,
    pub purchase_date: String,
    pub started_at: String,
    pub finished_at: String,
    pub progress_pages: i64,
    pub loaned_to: String,
    pub tags: String,
    pub created_at: String,
    pub updated_at: String,
}

impl Default for Book {
    fn default() -> Self {
        Self {
            id: None,
            work_key: None,
            edition_key: None,
            source: String::new(),
            source_id: String::new(),
            isbn: None,
            title: String::new(),
            authors: String::new(),
            publisher: String::new(),
            collection: String::new(),
            published_date: String::new(),
            format: String::new(),
            description: String::new(),
            subjects: String::new(),
            publish_year: None,
            pages: None,
            language: "fr".to_string(),
            cover_url: String::new(),
            cover_path: String::new(),
            status: "wishlist".to_string(),
            owned: false,
            rating: None,
            review: String::new(),
            liked: String::new(),
            disliked: String::new(),
            location: String::new(),
            purchase_price: None,
            purchase_date: String::new(),
            started_at: String::new(),
            finished_at: String::new(),
            progress_pages: 0,
            loaned_to: String::new(),
            tags: String::new(),
            created_at: String::new(),
            updated_at: String::new(),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct SearchResult {
    pub work_key: String,
    pub edition_key: Option<String>,
    pub source: String,
    pub source_id: String,
    pub isbn: Option<String>,
    /// Tous les ISBN d’éditions rattachées à cette même œuvre dans le
    /// catalogue. Champ temporaire, jamais écrit dans SQLite.
    #[serde(default)]
    pub alternate_isbns: Vec<String>,
    pub title: String,
    pub authors: String,
    pub publisher: String,
    pub collection: String,
    pub published_date: String,
    pub format: String,
    pub description: String,
    pub subjects: String,
    pub publish_year: Option<i64>,
    pub pages: Option<i64>,
    pub language: String,
    pub cover_url: String,
    pub edition_count: i64,
    pub relevance_score: f64,
    pub match_reasons: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NamedCount {
    pub name: String,
    pub count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardStats {
    pub total: i64,
    pub owned: i64,
    pub read: i64,
    pub reading: i64,
    pub wishlist: i64,
    pub abandoned: i64,
    pub average_rating: f64,
    pub pages_read: i64,
    pub profiled_books: i64,
    pub top_authors: Vec<NamedCount>,
    pub top_subjects: Vec<NamedCount>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Recommendation {
    pub book: SearchResult,
    pub score: f64,
    pub reasons: Vec<String>,
    pub warnings: Vec<String>,
}
