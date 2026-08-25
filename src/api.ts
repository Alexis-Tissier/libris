import { invoke } from "@tauri-apps/api/core";
import type { Book, DashboardStats, Recommendation, SearchResult } from "./types";

const isTauri = typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

async function invokeOrHttp<T>(command: string, args: Record<string, unknown> = {}): Promise<T> {
  if (isTauri) return invoke<T>(command, args);

  const response = await fetch(`/api/invoke/${encodeURIComponent(command)}`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(args)
  });

  if (!response.ok) {
    const message = (await response.text()).trim();
    throw new Error(message || `Erreur HTTP ${response.status}`);
  }

  const text = await response.text();
  return (text ? JSON.parse(text) : null) as T;
}

export const api = {
  runtime: isTauri ? "desktop" as const : "web" as const,

  listBooks(filter = "all", query = "") {
    return invokeOrHttp<Book[]>("list_books", { filter, query });
  },
  getBook(id: number) {
    return invokeOrHttp<Book | null>("get_book", { id });
  },
  saveBook(book: Book) {
    return invokeOrHttp<Book>("save_book", { book });
  },
  deleteBook(id: number) {
    return invokeOrHttp<void>("delete_book", { id });
  },
  stats() {
    return invokeOrHttp<DashboardStats>("get_stats");
  },
  search(query: string) {
    return invokeOrHttp<SearchResult[]>("search_catalog", { query });
  },
  enrich(book: SearchResult) {
    return invokeOrHttp<SearchResult>("enrich_search_result", { book });
  },
  resolveCover(coverUrl = "", isbn: string | null = null, source = "", sourceId = "") {
    return invokeOrHttp<string | null>("resolve_cover", { coverUrl, isbn, source, sourceId });
  },
  recommendations(mood = "", maxPages: number | null = null, genre = "auto", resultLimit = 24) {
    return invokeOrHttp<Recommendation[]>("get_recommendations", { mood, maxPages, genre, resultLimit });
  },
  saveFeedback(candidateKey: string, action: string) {
    return invokeOrHttp<void>("save_recommendation_feedback", { candidateKey, action });
  },
  databasePath() {
    return invokeOrHttp<string>("get_database_path");
  },
  exportData(path: string) {
    return invokeOrHttp<number>("export_data", { path });
  },
  importData(path: string) {
    return invokeOrHttp<number>("import_data", { path });
  },

  async exportWeb() {
    const response = await fetch("/api/export", { method: "GET" });
    if (!response.ok) {
      const message = (await response.text()).trim();
      throw new Error(message || `Erreur HTTP ${response.status}`);
    }
    const blob = await response.blob();
    const count = Number(response.headers.get("x-libris-count") ?? "0") || 0;
    return { blob, count };
  },

  async importWeb(file: File) {
    const response = await fetch("/api/import", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: await file.text()
    });
    if (!response.ok) {
      const message = (await response.text()).trim();
      throw new Error(message || `Erreur HTTP ${response.status}`);
    }
    const payload = await response.json() as { count?: number };
    return Number(payload.count ?? 0);
  }
};
