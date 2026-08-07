import { useEffect, useState } from "react";
import { api } from "../api";
import type { DashboardStats, Recommendation, SearchResult } from "../types";
import { Cover } from "../components/Cover";
import { Icon } from "../components/Icon";

const emptyStats: DashboardStats = {
  total: 0,
  owned: 0,
  read: 0,
  reading: 0,
  wishlist: 0,
  abandoned: 0,
  averageRating: 0,
  pagesRead: 0,
  profiledBooks: 0,
  topAuthors: [],
  topSubjects: []
};

const genreOptions = [
  ["auto", "Automatique"],
  ["adventure", "Aventure"],
  ["fantasy", "Fantastique et fantasy"],
  ["classics", "Romans classiques"],
  ["dystopia", "Dystopie"],
  ["science-fiction", "Science-fiction"],
  ["mystery", "Mystère et enquête"],
  ["horror", "Horreur et gothique"],
  ["history", "Roman historique et histoire"],
  ["philosophy", "Philosophie et société"],
  ["theatre-poetry", "Théâtre et poésie"],
  ["manga", "Manga"],
  ["mythology", "Mythologie et épopée"]
] as const;

export function ProfilePage({ refreshKey, onAdd }: { refreshKey: number; onAdd: (book: SearchResult) => void }) {
  const [stats, setStats] = useState<DashboardStats>(emptyStats);
  const [recommendations, setRecommendations] = useState<Recommendation[]>([]);
  const [mood, setMood] = useState("");
  const [genre, setGenre] = useState("auto");
  const [maxPages, setMaxPages] = useState("");
  const [resultLimit, setResultLimit] = useState("24");
  const [loading, setLoading] = useState(false);
  const [hasSearched, setHasSearched] = useState(false);
  const [error, setError] = useState("");

  useEffect(() => {
    let active = true;
    api.stats()
      .then((nextStats) => {
        if (active) setStats(nextStats);
      })
      .catch((reason) => {
        if (active) setError(String(reason));
      });
    return () => { active = false; };
  }, [refreshKey]);

  async function loadRecommendations() {
    setLoading(true);
    setHasSearched(true);
    setError("");
    try {
      const pageLimit = maxPages.trim() ? Number(maxPages) : null;
      setRecommendations(await api.recommendations(
        mood.trim(),
        pageLimit,
        genre,
        Number(resultLimit)
      ));
    } catch (reason) {
      setRecommendations([]);
      setError(String(reason));
    } finally {
      setLoading(false);
    }
  }

  async function dismiss(item: Recommendation) {
    const key = item.book.workKey || item.book.isbn || `${item.book.title}::${item.book.authors}`;
    await api.saveFeedback(key, "dismiss");
    setRecommendations((current) => current.filter((candidate) => candidate !== item));
  }

  const enoughData = stats.averageRating > 0;

  return (
    <div className="page">
      <header className="page-header">
        <div>
          <span className="eyebrow">Profil lecteur</span>
          <h1>Pour vous</h1>
          <p>Libris part de vos notes et de vos affinités, puis écarte les œuvres déjà présentes dans votre bibliothèque.</p>
        </div>
      </header>

      <section className="taste-layout">
        <article className="taste-card primary">
          <span className="eyebrow light">Votre empreinte littéraire</span>
          <h2>{stats.total ? `${stats.total} livres façonnent déjà votre profil` : "Votre profil se construira avec vos lectures"}</h2>
          <p>{stats.averageRating ? `Votre note moyenne est de ${stats.averageRating}/10.` : "Notez quelques livres pour commencer à distinguer ce qui vous attire vraiment."}</p>
          <div className="taste-stat-row">
            <div><strong>{stats.read}</strong><span>lus</span></div>
            <div><strong>{stats.topAuthors.length}</strong><span>auteurs marquants</span></div>
            <div><strong>{stats.topSubjects.length}</strong><span>thèmes détectés</span></div>
          </div>
        </article>

        <article className="taste-card">
          <span className="eyebrow">Auteurs qui comptent</span>
          <div className="rank-list">
            {stats.topAuthors.length
              ? stats.topAuthors.map((item, index) => (
                <div key={item.name}>
                  <span>{String(index + 1).padStart(2, "0")}</span>
                  <strong>{item.name}</strong>
                  <small>{item.count} livre{item.count > 1 ? "s" : ""} apprécié{item.count > 1 ? "s" : ""}</small>
                </div>
              ))
              : <p className="muted-copy">Aucun auteur suffisamment évalué pour l’instant.</p>}
          </div>
        </article>
      </section>

      <section className="taste-subjects">
        <div className="section-row-heading">
          <div>
            <span className="eyebrow">Affinités</span>
            <h2>Genres et thèmes récurrents</h2>
            <p className="muted-copy">
              Analyse de {stats.profiledBooks} livre{stats.profiledBooks > 1 ? "s" : ""} possédé{stats.profiledBooks > 1 ? "s" : ""}. Les séries comptent tome par tome et un même livre peut apparaître dans plusieurs catégories.
            </p>
          </div>
        </div>
        <div className="tag-cloud">
          {stats.topSubjects.length
            ? stats.topSubjects.map((item) => <span key={item.name}>{item.name}<small>{item.count}</small></span>)
            : <span className="muted-copy">Ils apparaîtront ici dès que les notices contiennent assez d’informations.</span>}
        </div>
      </section>

      <section className="recommendation-lab">
        <div className="recommendation-lab-copy">
          <span className="eyebrow light">Choisir maintenant</span>
          <h2>Quelle lecture cherchez-vous aujourd’hui ?</h2>
          <p>Choisissez un genre pour obtenir une sélection vraiment ciblée, ou laissez Libris équilibrer vos auteurs, séries et notes. Seules des éditions françaises sont proposées.</p>
        </div>
        <div className="recommendation-controls">
          <label>
            <span>Genre</span>
            <select value={genre} onChange={(event) => setGenre(event.target.value)}>
              {genreOptions.map(([value, label]) => <option value={value} key={value}>{label}</option>)}
            </select>
          </label>
          <label>
            <span>Envie actuelle</span>
            <input
              value={mood}
              onChange={(event) => setMood(event.target.value)}
              placeholder="court, sombre, captivant…"
            />
          </label>
          <label>
            <span>Longueur maximale</span>
            <div className="input-suffix">
              <input
                type="number"
                min="1"
                value={maxPages}
                onChange={(event) => setMaxPages(event.target.value)}
                placeholder="Sans limite"
              />
              <span>pages</span>
            </div>
          </label>
          <label>
            <span>Propositions</span>
            <select value={resultLimit} onChange={(event) => setResultLimit(event.target.value)}>
              <option value="12">12</option>
              <option value="24">24</option>
              <option value="36">36</option>
            </select>
          </label>
          <button className="button button-light" onClick={() => void loadRecommendations()} disabled={loading || !enoughData}>
            <Icon name="sparkles" size={18} />
            {loading ? "Analyse…" : "Me recommander"}
          </button>
        </div>
      </section>

      {!enoughData ? (
        <div className="info-panel">
          <Icon name="book" size={23} />
          <div>
            <strong>Notez au moins un livre</strong>
            <p>Libris se base sur vos évaluations explicites, puis utilise l’ensemble de vos lectures pour comprendre les genres présents.</p>
          </div>
        </div>
      ) : null}

      {error ? <div className="error-inline">{error}</div> : null}

      {hasSearched && !loading && !error && !recommendations.length ? (
        <div className="info-panel">
          <Icon name="book" size={23} />
          <div>
            <strong>Aucune proposition française suffisamment fiable</strong>
            <p>Essaie un autre genre, une envie plus générale ou retire la limite de pages.</p>
          </div>
        </div>
      ) : null}

      {recommendations.length ? (
        <section>
          <div className="section-row-heading recommendation-heading">
            <div>
              <span className="eyebrow">Sélection personnalisée</span>
              <h2>Vos recommandations</h2>
              <p className="muted-copy">Éditions françaises uniquement, une proposition par œuvre et aucune œuvre déjà présente sous un autre ISBN.</p>
            </div>
          </div>
          <div className="recommendation-grid">
            {recommendations.map((item) => (
              <article className="recommendation-card" key={`${item.book.workKey}-${item.book.title}-${item.book.authors}`}>
                <div className="score-badge"><strong>{item.score}%</strong><span>compatibilité</span></div>
                <Cover
                  src={item.book.coverUrl}
                  title={item.book.title}
                  isbn={item.book.isbn}
                  source={item.book.source}
                  sourceId={item.book.sourceId}
                />
                <div className="recommendation-body">
                  <h3>{item.book.title}</h3>
                  <p className="book-author">{item.book.authors}</p>
                  <ul>{item.reasons.map((reason) => <li key={reason}><Icon name="check" size={14} />{reason}</li>)}</ul>
                  {item.warnings.map((warning) => <p className="warning-copy" key={warning}>{warning}</p>)}
                  <div className="recommendation-actions">
                    <button className="button button-primary" onClick={() => onAdd(item.book)}><Icon name="plus" size={16} />Ajouter</button>
                    <button className="icon-button" onClick={() => void dismiss(item)} aria-label="Ne plus proposer"><Icon name="close" size={17} /></button>
                  </div>
                </div>
              </article>
            ))}
          </div>
        </section>
      ) : null}
    </div>
  );
}
