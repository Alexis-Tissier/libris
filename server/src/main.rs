use axum::{
    body::Bytes,
    extract::{DefaultBodyLimit, Path},
    http::{header, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde::de::DeserializeOwned;
use serde_json::{json, Value};
use std::{env, fs, net::SocketAddr, path::PathBuf, time::{SystemTime, UNIX_EPOCH}};
use tower_http::services::{ServeDir, ServeFile};

#[path = "../../src-tauri/src/models.rs"]
mod models;
#[path = "../../src-tauri/src/db.rs"]
mod db;
#[path = "../../src-tauri/src/openlibrary.rs"]
mod openlibrary;
#[path = "../../src-tauri/src/catalog.rs"]
mod catalog;
#[path = "../../src-tauri/src/recommender.rs"]
mod recommender;

use models::{Book, SearchResult};

type ApiError = (StatusCode, String);

fn bad_request(message: impl Into<String>) -> ApiError {
    (StatusCode::BAD_REQUEST, message.into())
}

fn internal_error(message: impl Into<String>) -> ApiError {
    (StatusCode::INTERNAL_SERVER_ERROR, message.into())
}

fn value_arg<'a>(args: &'a Value, key: &str) -> Option<&'a Value> {
    args.as_object().and_then(|map| map.get(key))
}

fn string_arg(args: &Value, key: &str) -> Option<String> {
    value_arg(args, key).and_then(Value::as_str).map(ToOwned::to_owned)
}

fn i64_arg(args: &Value, key: &str) -> Result<i64, ApiError> {
    value_arg(args, key)
        .and_then(Value::as_i64)
        .ok_or_else(|| bad_request(format!("Argument {key} manquant ou invalide")))
}

fn parse_arg<T: DeserializeOwned>(args: &Value, key: &str) -> Result<T, ApiError> {
    let value = value_arg(args, key)
        .cloned()
        .ok_or_else(|| bad_request(format!("Argument {key} manquant")))?;
    serde_json::from_value(value).map_err(|error| bad_request(error.to_string()))
}

fn into_json<T: serde::Serialize>(result: Result<T, String>) -> Result<Json<Value>, ApiError> {
    let value = result.map_err(bad_request)?;
    serde_json::to_value(value)
        .map(Json)
        .map_err(|error| internal_error(error.to_string()))
}

async fn health() -> Json<Value> {
    let database = db::database_path()
        .map(|path| path.to_string_lossy().to_string())
        .unwrap_or_else(|_| "indisponible".to_string());
    Json(json!({
        "status": "ok",
        "service": "libris",
        "version": "0.5.0-web",
        "database": database
    }))
}

async fn invoke(Path(command): Path<String>, Json(args): Json<Value>) -> Result<Json<Value>, ApiError> {
    match command.as_str() {
        "list_books" => into_json(db::list_books(
            string_arg(&args, "filter").as_deref(),
            string_arg(&args, "query").as_deref(),
        )),
        "get_book" => into_json(db::get_book(i64_arg(&args, "id")?)),
        "save_book" => into_json(db::save_book(parse_arg::<Book>(&args, "book")?)),
        "delete_book" => {
            db::delete_book(i64_arg(&args, "id")?).map_err(bad_request)?;
            Ok(Json(Value::Null))
        }
        "get_stats" => into_json(db::stats()),
        "search_catalog" => {
            let query = string_arg(&args, "query").ok_or_else(|| bad_request("Argument query manquant"))?;
            into_json(catalog::search(&query, 60).await)
        }
        "enrich_search_result" => {
            let book = parse_arg::<SearchResult>(&args, "book")?;
            into_json::<SearchResult>(Ok(catalog::enrich(book).await))
        }
        "resolve_cover" => {
            let cover_url = string_arg(&args, "coverUrl").unwrap_or_default();
            let isbn = value_arg(&args, "isbn").and_then(Value::as_str).map(ToOwned::to_owned);
            let source = string_arg(&args, "source").unwrap_or_default();
            let source_id = string_arg(&args, "sourceId").unwrap_or_default();
            into_json(catalog::resolve_cover(&cover_url, isbn.as_deref(), &source, &source_id).await)
        }
        "get_recommendations" => {
            let mood = string_arg(&args, "mood").unwrap_or_default();
            let max_pages = value_arg(&args, "maxPages").and_then(Value::as_i64);
            let genre = string_arg(&args, "genre").unwrap_or_else(|| "auto".to_string());
            let result_limit = value_arg(&args, "resultLimit")
                .and_then(Value::as_u64)
                .map(|value| value as usize);
            into_json(recommender::recommend(&mood, max_pages, &genre, result_limit).await)
        }
        "save_recommendation_feedback" => {
            let candidate_key = string_arg(&args, "candidateKey")
                .ok_or_else(|| bad_request("Argument candidateKey manquant"))?;
            let action = string_arg(&args, "action").ok_or_else(|| bad_request("Argument action manquant"))?;
            db::save_feedback(&candidate_key, &action).map_err(bad_request)?;
            Ok(Json(Value::Null))
        }
        "get_database_path" => into_json(db::database_path().map(|path| path.to_string_lossy().to_string())),
        "export_data" | "import_data" => Err(bad_request(
            "Sur le web, utilisez les routes d’import/export dédiées de Libris.",
        )),
        _ => Err((StatusCode::NOT_FOUND, format!("Commande Libris inconnue : {command}"))),
    }
}

fn temporary_json_path(prefix: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or_default();
    env::temp_dir().join(format!("libris-{prefix}-{}-{stamp}.json", std::process::id()))
}

async fn export_data() -> Result<Response, ApiError> {
    let path = temporary_json_path("export");
    let count = db::export_json(&path).map_err(bad_request)?;
    let bytes = fs::read(&path).map_err(|error| internal_error(error.to_string()))?;
    let _ = fs::remove_file(&path);

    let mut response = bytes.into_response();
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/json; charset=utf-8"),
    );
    response.headers_mut().insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_static("attachment; filename=\"libris-export.json\""),
    );
    if let Ok(value) = HeaderValue::from_str(&count.to_string()) {
        response.headers_mut().insert("x-libris-count", value);
    }
    Ok(response)
}

async fn import_data(body: Bytes) -> Result<Json<Value>, ApiError> {
    if body.is_empty() {
        return Err(bad_request("Fichier d’import vide"));
    }

    let path = temporary_json_path("import");
    fs::write(&path, &body).map_err(|error| internal_error(error.to_string()))?;
    let result = db::import_json(&path);
    let _ = fs::remove_file(&path);
    let count = result.map_err(bad_request)?;
    Ok(Json(json!({ "count": count })))
}

async fn shutdown_signal() {
    let ctrl_c = async {
        let _ = tokio::signal::ctrl_c().await;
    };

    #[cfg(unix)]
    let terminate = async {
        if let Ok(mut signal) = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            signal.recv().await;
        }
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}

#[tokio::main]
async fn main() {
    if let Err(error) = db::init() {
        eprintln!("Libris database init error: {error}");
        std::process::exit(1);
    }

    let bind = env::var("LIBRIS_BIND").unwrap_or_else(|_| "0.0.0.0:8030".to_string());
    let address: SocketAddr = bind.parse().expect("LIBRIS_BIND invalide");
    let dist = PathBuf::from(env::var("LIBRIS_DIST_DIR").unwrap_or_else(|_| "/app/dist".to_string()));
    let index = dist.join("index.html");

    let app = Router::new()
        .route("/api/health", get(health))
        .route("/api/invoke/{command}", post(invoke))
        .route("/api/export", get(export_data))
        .route("/api/import", post(import_data))
        .layer(DefaultBodyLimit::max(32 * 1024 * 1024))
        .fallback_service(ServeDir::new(&dist).not_found_service(ServeFile::new(index)));

    let listener = tokio::net::TcpListener::bind(address)
        .await
        .expect("Impossible d’écouter le port Libris");
    println!("Libris web écoute sur http://{address}");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .expect("Erreur serveur Libris");
}
