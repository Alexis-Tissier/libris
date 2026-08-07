import { useEffect, useState } from "react";
import { api } from "../api";
import type { Book, DashboardStats, PageId } from "../types";
import { Cover } from "../components/Cover";
import { Icon } from "../components/Icon";

const emptyStats: DashboardStats = {
  total: 0, owned: 0, read: 0, reading: 0, wishlist: 0, abandoned: 0,
  averageRating: 0, pagesRead: 0, profiledBooks: 0, topAuthors: [], topSubjects: []
};

export function HomePage({ refreshKey, onNavigate, onOpenBook }: { refreshKey: number; onNavigate: (page: PageId) => void; onOpenBook: (book: Book) => void }) {
  const [stats, setStats] = useState<DashboardStats>(emptyStats);
  const [books, setBooks] = useState<Book[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    setLoading(true);
    Promise.all([api.stats(), api.listBooks("all", "")])
      .then(([nextStats, nextBooks]) => { setStats(nextStats); setBooks(nextBooks); })
      .finally(() => setLoading(false));
  }, [refreshKey]);

  const current = books.find((book) => book.status === "reading");
  const recent = books.slice(0, 6);

  return (
    <div className="page page-home">
      <header className="page-header">
        <div>
          <span className="eyebrow">Votre bibliothèque personnelle</span>
          <h1>Votre bibliothèque.</h1>
          <p>Retrouvez vos lectures, votre collection physique et la prochaine histoire qui mérite votre temps.</p>
        </div>
        <button className="button button-primary" onClick={() => onNavigate("discover")}><Icon name="plus" size={18} />Ajouter un livre</button>
      </header>

      <section className="metrics-grid" aria-label="Résumé de la bibliothèque">
        <article className="metric-card"><span>Possédés</span><strong>{stats.owned}</strong><small>exemplaires physiques</small></article>
        <article className="metric-card"><span>Lus</span><strong>{stats.read}</strong><small>{stats.pagesRead.toLocaleString("fr-FR")} pages terminées</small></article>
        <article className="metric-card"><span>À lire</span><strong>{stats.wishlist}</strong><small>dans votre sélection</small></article>
        <article className="metric-card"><span>Note moyenne</span><strong>{stats.averageRating ? `${stats.averageRating}/10` : "—"}</strong><small>sur les livres notés</small></article>
      </section>

      <section className={`reading-hero ${current ? "has-book" : "is-empty"}`}>
        {current ? (
          <>
            <div className="reading-hero-copy">
              <span className="eyebrow light">Lecture en cours</span>
              <h2>{current.title}</h2>
              <p>{current.authors}</p>
              <div className="reading-progress-label"><span>Page {current.progressPages}{current.pages ? ` sur ${current.pages}` : ""}</span><strong>{current.pages ? Math.round((current.progressPages / current.pages) * 100) : 0}%</strong></div>
              <div className="reading-progress"><span style={{ width: `${current.pages ? Math.min(100, Math.round((current.progressPages / current.pages) * 100)) : 0}%` }} /></div>
              <button className="button button-light" onClick={() => onOpenBook(current)}>Mettre à jour la lecture <Icon name="arrowRight" size={17} /></button>
            </div>
            <Cover src={current.coverPath || current.coverUrl} title={current.title} isbn={current.isbn} source={current.source} sourceId={current.sourceId} className="reading-cover" />
          </>
        ) : (
          <div className="reading-empty-copy">
            <span className="eyebrow light">Votre prochaine histoire</span>
            <h2>Aucune lecture en cours</h2>
            <p>Choisissez un livre de votre sélection ou découvrez une nouvelle lecture adaptée à vos goûts.</p>
            <div className="button-row">
              <button className="button button-light" onClick={() => onNavigate("discover")}>Découvrir un livre</button>
              {books.length ? <button className="button button-dark-ghost" onClick={() => onNavigate("library")}>Voir ma bibliothèque</button> : null}
            </div>
          </div>
        )}
      </section>

      <div className="section-row-heading">
        <div><span className="eyebrow">Collection</span><h2>Ajouts récents</h2></div>
        {books.length ? <button className="text-button" onClick={() => onNavigate("library")}>Tout voir <Icon name="arrowRight" size={16} /></button> : null}
      </div>

      {loading ? (
        <div className="recent-grid">{Array.from({ length: 4 }).map((_, index) => <div className="recent-card skeleton" key={index} />)}</div>
      ) : recent.length ? (
        <div className="recent-grid">
          {recent.map((book) => (
            <button className="recent-card" key={book.id} onClick={() => onOpenBook(book)}>
              <Cover src={book.coverPath || book.coverUrl} title={book.title} isbn={book.isbn} source={book.source} sourceId={book.sourceId} />
              <div><h3>{book.title}</h3><p>{book.authors}</p><span>{book.status === "read" ? "Lu" : book.status === "reading" ? "En cours" : book.status === "abandoned" ? "Abandonné" : "À lire"}</span></div>
            </button>
          ))}
        </div>
      ) : (
        <div className="empty-panel">
          <div className="empty-icon"><Icon name="library" size={28} /></div>
          <h3>Votre bibliothèque est prête</h3>
          <p>Recherchez un titre, un auteur ou un ISBN pour ajouter votre premier livre physique.</p>
          <button className="button button-primary" onClick={() => onNavigate("discover")}>Ajouter mon premier livre</button>
        </div>
      )}
    </div>
  );
}
