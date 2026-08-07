import { useEffect, useMemo, useState } from "react";
import { api } from "../api";
import type { Book } from "../types";
import { BookCard } from "../components/BookCard";
import { Icon } from "../components/Icon";

const filters = [
  ["all", "Tous"], ["owned", "Possédés"], ["wishlist", "À lire"], ["reading", "En cours"], ["read", "Lus"], ["abandoned", "Abandonnés"]
] as const;

export function LibraryPage({ refreshKey, onOpenBook, onDiscover }: { refreshKey: number; onOpenBook: (book: Book) => void; onDiscover: () => void }) {
  const [books, setBooks] = useState<Book[]>([]);
  const [filter, setFilter] = useState<(typeof filters)[number][0]>("all");
  const [query, setQuery] = useState("");
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    setLoading(true);
    api.listBooks(filter, query).then(setBooks).finally(() => setLoading(false));
  }, [filter, query, refreshKey]);

  const countLabel = useMemo(() => `${books.length} livre${books.length > 1 ? "s" : ""}`, [books.length]);

  return (
    <div className="page">
      <header className="page-header">
        <div><span className="eyebrow">Collection physique et historique</span><h1>Bibliothèque</h1><p>Vos livres possédés, lus, prêtés ou simplement gardés pour plus tard.</p></div>
        <button className="button button-primary" onClick={onDiscover}><Icon name="plus" size={18} />Ajouter</button>
      </header>

      <div className="library-toolbar">
        <div className="search-field compact"><Icon name="search" size={18} /><input value={query} onChange={(event) => setQuery(event.target.value)} placeholder="Rechercher dans votre bibliothèque…" />{query ? <button onClick={() => setQuery("")}><Icon name="close" size={16} /></button> : null}</div>
        <span className="result-count">{countLabel}</span>
      </div>

      <div className="filter-tabs" role="tablist">
        {filters.map(([value, label]) => <button key={value} className={filter === value ? "is-active" : ""} onClick={() => setFilter(value)}>{label}</button>)}
      </div>

      {loading ? (
        <div className="book-grid">{Array.from({ length: 8 }).map((_, index) => <div className="book-card skeleton" key={index} />)}</div>
      ) : books.length ? (
        <div className="book-grid">{books.map((book) => <BookCard key={book.id} book={book} onOpen={onOpenBook} />)}</div>
      ) : (
        <div className="empty-panel large"><div className="empty-icon"><Icon name="book" size={30} /></div><h3>Aucun livre ici</h3><p>{query ? "Aucun résultat ne correspond à cette recherche." : "Cette partie de votre bibliothèque est encore vide."}</p>{!query ? <button className="button button-primary" onClick={onDiscover}>Découvrir des livres</button> : null}</div>
      )}
    </div>
  );
}
