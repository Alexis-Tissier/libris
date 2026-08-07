mod catalog;
mod db;
mod models;
mod openlibrary;
mod recommender;

use models::{Book, DashboardStats, Recommendation, SearchResult};
use std::path::PathBuf;

#[tauri::command]
fn list_books(filter: Option<String>, query: Option<String>) -> Result<Vec<Book>, String> {
    db::list_books(filter.as_deref(), query.as_deref())
}

#[tauri::command]
fn get_book(id: i64) -> Result<Option<Book>, String> {
    db::get_book(id)
}

#[tauri::command]
fn save_book(book: Book) -> Result<Book, String> {
    db::save_book(book)
}

#[tauri::command]
fn delete_book(id: i64) -> Result<(), String> {
    db::delete_book(id)
}

#[tauri::command]
fn get_stats() -> Result<DashboardStats, String> {
    db::stats()
}

#[tauri::command]
async fn search_catalog(query: String) -> Result<Vec<SearchResult>, String> {
    catalog::search(&query, 60).await
}

#[tauri::command]
async fn enrich_search_result(book: SearchResult) -> Result<SearchResult, String> {
    Ok(catalog::enrich(book).await)
}

#[tauri::command]
async fn resolve_cover(
    cover_url: String,
    isbn: Option<String>,
    source: String,
    source_id: String,
) -> Result<Option<String>, String> {
    catalog::resolve_cover(&cover_url, isbn.as_deref(), &source, &source_id).await
}

#[tauri::command]
async fn get_recommendations(
    mood: Option<String>,
    max_pages: Option<i64>,
    genre: Option<String>,
    result_limit: Option<usize>,
) -> Result<Vec<Recommendation>, String> {
    recommender::recommend(
        mood.as_deref().unwrap_or(""),
        max_pages,
        genre.as_deref().unwrap_or("auto"),
        result_limit,
    )
    .await
}

#[tauri::command]
fn save_recommendation_feedback(candidate_key: String, action: String) -> Result<(), String> {
    db::save_feedback(&candidate_key, &action)
}

#[tauri::command]
fn get_database_path() -> Result<String, String> {
    db::database_path().map(|path| path.to_string_lossy().to_string())
}

#[tauri::command]
fn export_data(path: String) -> Result<usize, String> {
    db::export_json(&PathBuf::from(path))
}

#[tauri::command]
fn import_data(path: String) -> Result<usize, String> {
    db::import_json(&PathBuf::from(path))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    if let Err(error) = db::init() {
        eprintln!("Libris database init error: {error}");
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            list_books,
            get_book,
            save_book,
            delete_book,
            get_stats,
            search_catalog,
            enrich_search_result,
            resolve_cover,
            get_recommendations,
            save_recommendation_feedback,
            get_database_path,
            export_data,
            import_data
        ])
        .run(tauri::generate_context!())
        .expect("error while running Libris");
}
