import { useEffect, useState } from "react";
import { api } from "./api";
import type { Book, PageId, SearchResult } from "./types";
import { emptyBook } from "./types";
import { Shell } from "./components/Shell";
import { BookDrawer } from "./components/BookDrawer";
import { Toast } from "./components/Toast";
import { HomePage } from "./pages/HomePage";
import { LibraryPage } from "./pages/LibraryPage";
import { DiscoverPage } from "./pages/DiscoverPage";
import { ProfilePage } from "./pages/ProfilePage";
import { DataPage } from "./pages/DataPage";

interface ToastState { message: string; kind: "success" | "error"; }

export default function App() {
  const [page, setPage] = useState<PageId>("home");
  const [drawerBook, setDrawerBook] = useState<Book | null>(null);
  const [refreshKey, setRefreshKey] = useState(0);
  const [toast, setToast] = useState<ToastState | null>(null);

  useEffect(() => {
    if (!toast) return;
    const timer = window.setTimeout(() => setToast(null), 4200);
    return () => window.clearTimeout(timer);
  }, [toast]);

  function notify(message: string, kind: "success" | "error" = "success") {
    setToast({ message, kind });
  }

  function navigate(next: PageId) {
    setPage(next);
  }

  function openBook(book: Book) {
    setDrawerBook({ ...book });
  }

  async function addSearchResult(result: SearchResult) {
    setDrawerBook(emptyBook(result));
    try {
      const enriched = await api.enrich(result);
      setDrawerBook((current) => current?.id === null && current.workKey === result.workKey ? { ...current, description: enriched.description, subjects: enriched.subjects || current.subjects } : current);
    } catch {
      // La fiche reste utilisable même si l’enrichissement détaillé échoue.
    }
  }

  async function saveBook(book: Book) {
    try {
      const saved = await api.saveBook(book);
      setDrawerBook(null);
      setRefreshKey((value) => value + 1);
      notify(`« ${saved.title} » a été enregistré.`);
    } catch (reason) {
      notify(String(reason), "error");
      throw reason;
    }
  }

  async function deleteBook(book: Book) {
    if (!book.id) return;
    try {
      await api.deleteBook(book.id);
      setDrawerBook(null);
      setRefreshKey((value) => value + 1);
      notify(`« ${book.title} » a été supprimé.`);
    } catch (reason) {
      notify(String(reason), "error");
      throw reason;
    }
  }

  return (
    <>
      <Shell page={page} onNavigate={navigate}>
        {page === "home" ? <HomePage refreshKey={refreshKey} onNavigate={navigate} onOpenBook={openBook} /> : null}
        {page === "library" ? <LibraryPage refreshKey={refreshKey} onOpenBook={openBook} onDiscover={() => navigate("discover")} /> : null}
        {page === "discover" ? <DiscoverPage onAdd={(book) => void addSearchResult(book)} /> : null}
        {page === "profile" ? <ProfilePage refreshKey={refreshKey} onAdd={(book) => void addSearchResult(book)} /> : null}
        {page === "data" ? <DataPage onImported={() => setRefreshKey((value) => value + 1)} notify={notify} /> : null}
      </Shell>

      {drawerBook ? <BookDrawer initialBook={drawerBook} onClose={() => setDrawerBook(null)} onSave={saveBook} onDelete={deleteBook} /> : null}
      {toast ? <Toast message={toast.message} kind={toast.kind} onClose={() => setToast(null)} /> : null}
    </>
  );
}
