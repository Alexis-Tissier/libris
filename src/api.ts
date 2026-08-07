import { invoke } from "@tauri-apps/api/core";
import type { Book, DashboardStats, Recommendation, SearchResult } from "./types";

export const api = {
  listBooks(filter = "all", query = "") {
    return invoke<Book[]>("list_books", { filter, query });
  },
  getBook(id: number) {
    return invoke<Book | null>("get_book", { id });
  },
  saveBook(book: Book) {
    return invoke<Book>("save_book", { book });
  },
  deleteBook(id: number) {
    return invoke<void>("delete_book", { id });
  },
  stats() {
    return invoke<DashboardStats>("get_stats");
  },
  search(query: string) {
    return invoke<SearchResult[]>("search_catalog", { query });
  },
  enrich(book: SearchResult) {
    return invoke<SearchResult>("enrich_search_result", { book });
  },
  resolveCover(coverUrl = "", isbn: string | null = null, source = "", sourceId = "") {
    return invoke<string | null>("resolve_cover", { coverUrl, isbn, source, sourceId });
  },
  recommendations(mood = "", maxPages: number | null = null, genre = "auto", resultLimit = 24) {
    return invoke<Recommendation[]>("get_recommendations", { mood, maxPages, genre, resultLimit });
  },
  saveFeedback(candidateKey: string, action: string) {
    return invoke<void>("save_recommendation_feedback", { candidateKey, action });
  },
  databasePath() {
    return invoke<string>("get_database_path");
  },
  exportData(path: string) {
    return invoke<number>("export_data", { path });
  },
  importData(path: string) {
    return invoke<number>("import_data", { path });
  }
};
