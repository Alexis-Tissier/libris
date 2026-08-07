import { FormEvent, useState } from "react";
import { api } from "../api";
import type { SearchResult } from "../types";
import { SearchBookCard } from "../components/BookCard";
import { Icon } from "../components/Icon";

const suggestions = [
  "Stefan Zweig",
  "Pauline Alexandre Dumas Belin Gallimard Classico Lycée",
  "classiques courts",
  "dystopie",
  "aventure historique"
];

export function DiscoverPage({ onAdd }: { onAdd: (book: SearchResult) => void }) {
  const [query, setQuery] = useState("");
  const [results, setResults] = useState<SearchResult[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");
  const [searched, setSearched] = useState(false);

  async function runSearch(value = query) {
    const clean = value.trim();
    if (!clean) return;
    setQuery(clean);
    setLoading(true);
    setError("");
    setSearched(true);
    try {
      setResults(await api.search(clean));
    } catch (reason) {
      setResults([]);
      setError(String(reason));
    } finally {
      setLoading(false);
    }
  }

  function submit(event: FormEvent) {
    event.preventDefault();
    void runSearch();
  }

  const resultLabel = results.length === 1 ? "1 édition trouvée" : `${results.length} éditions trouvées`;

  return (
    <div className="page discover-page">
      <header className="discover-header">
        <span className="eyebrow">Éditions physiques françaises et internationales</span>
        <h1>Retrouvez exactement votre édition.</h1>
        <p>
          Saisissez le titre, l’auteur, l’éditeur, la collection ou l’ISBN imprimé au dos du livre.
          Libris croise la BnF, Google Books et Open Library sans fusionner les éditions différentes.
        </p>
        <form className="discover-search" onSubmit={submit}>
          <Icon name="search" size={22} />
          <input
            value={query}
            onChange={(event) => setQuery(event.target.value)}
            placeholder="Ex. Pauline Alexandre Dumas Belin Gallimard Classico Lycée"
            autoFocus
          />
          {query ? <button type="button" className="clear-search" onClick={() => setQuery("")}><Icon name="close" size={18} /></button> : null}
          <button className="button button-primary" disabled={loading || !query.trim()}>{loading ? "Recherche…" : "Rechercher"}</button>
        </form>
        <div className="suggestion-row"><span>Essayez :</span>{suggestions.map((suggestion) => <button key={suggestion} onClick={() => void runSearch(suggestion)}>{suggestion}</button>)}</div>
      </header>

      {loading ? (
        <section>
          <div className="section-row-heading">
            <div><span className="eyebrow">BnF · Google Books · Open Library</span><h2>Recherche des éditions</h2></div>
            <span className="muted-copy">Les trois catalogues sont interrogés en parallèle.</span>
          </div>
          <div className="search-results-grid">{Array.from({ length: 8 }).map((_, index) => <div className="search-book-card skeleton" key={index} />)}</div>
        </section>
      ) : error ? (
        <div className="error-panel">
          <div className="empty-icon"><Icon name="x" size={25} /></div>
          <h3>La recherche n’a pas abouti</h3>
          <p>{error}</p>
          <button className="button button-secondary" onClick={() => void runSearch()}><Icon name="refresh" size={17} />Réessayer</button>
        </div>
      ) : results.length ? (
        <section>
          <div className="section-row-heading">
            <div><span className="eyebrow">Résultats classés par édition</span><h2>{resultLabel}</h2></div>
            <span className="muted-copy">Vérifiez surtout l’éditeur, la collection, l’ISBN et la couverture.</span>
          </div>
          <div className="catalog-legend">
            <span><strong>BnF</strong> éditions françaises</span>
            <span><strong>Google Books</strong> ISBN et éditeurs</span>
            <span><strong>Open Library</strong> catalogue complémentaire</span>
          </div>
          <div className="search-results-grid">
            {results.map((book, index) => (
              <SearchBookCard
                key={`${book.source}-${book.sourceId}-${book.isbn ?? "sans-isbn"}-${index}`}
                book={book}
                onAdd={onAdd}
              />
            ))}
          </div>
        </section>
      ) : searched ? (
        <div className="empty-panel large">
          <div className="empty-icon"><Icon name="search" size={28} /></div>
          <h3>Aucune édition trouvée</h3>
          <p>Essayez avec l’ISBN exact, puis avec le titre, l’auteur, l’éditeur et la collection.</p>
        </div>
      ) : (
        <section className="discover-intro-grid">
          <article className="discover-feature featured"><span className="feature-number">01</span><div><h3>Chaque édition reste distincte</h3><p>Deux livres portant le même titre ne sont regroupés que lorsqu’ils partagent le même ISBN.</p></div></article>
          <article className="discover-feature"><span className="feature-number">02</span><div><h3>La France passe en priorité</h3><p>La BnF complète les catalogues mondiaux pour retrouver les éditions scolaires et collections françaises.</p></div></article>
          <article className="discover-feature"><span className="feature-number">03</span><div><h3>Le résultat explique son classement</h3><p>Les correspondances d’éditeur, de collection et d’ISBN sont affichées directement sur la fiche.</p></div></article>
        </section>
      )}
    </div>
  );
}
