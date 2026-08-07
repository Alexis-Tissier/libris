import { useEffect, useMemo, useState } from "react";
import type { Book, BookStatus } from "../types";
import { Cover } from "./Cover";
import { Icon } from "./Icon";

const statuses: { value: BookStatus; label: string; description: string }[] = [
  { value: "wishlist", label: "À lire", description: "Dans votre sélection" },
  { value: "reading", label: "En cours", description: "Lecture actuelle" },
  { value: "read", label: "Lu", description: "Terminé" },
  { value: "abandoned", label: "Abandonné", description: "Interrompu" }
];

function numberValue(value: string): number | null {
  if (!value.trim()) return null;
  const parsed = Number(value.replace(",", "."));
  return Number.isFinite(parsed) ? parsed : null;
}

export function BookDrawer({
  initialBook,
  onClose,
  onSave,
  onDelete
}: {
  initialBook: Book;
  onClose: () => void;
  onSave: (book: Book) => Promise<void>;
  onDelete: (book: Book) => Promise<void>;
}) {
  const [book, setBook] = useState<Book>(initialBook);
  const [saving, setSaving] = useState(false);
  const [deleting, setDeleting] = useState(false);

  useEffect(() => setBook(initialBook), [initialBook]);

  const progress = useMemo(() => {
    if (!book.pages || book.pages <= 0) return 0;
    return Math.min(100, Math.round((book.progressPages / book.pages) * 100));
  }, [book.pages, book.progressPages]);

  function patch<K extends keyof Book>(key: K, value: Book[K]) {
    setBook((current) => ({ ...current, [key]: value }));
  }

  async function save() {
    setSaving(true);
    try {
      await onSave(book);
    } finally {
      setSaving(false);
    }
  }

  async function remove() {
    if (!book.id || !window.confirm(`Supprimer « ${book.title} » de Libris ?`)) return;
    setDeleting(true);
    try {
      await onDelete(book);
    } finally {
      setDeleting(false);
    }
  }

  return (
    <div className="drawer-backdrop" onMouseDown={(event) => event.target === event.currentTarget && onClose()}>
      <section className="book-drawer" role="dialog" aria-modal="true" aria-label="Fiche du livre">
        <header className="drawer-header">
          <div>
            <span className="eyebrow">{book.id ? "Votre exemplaire" : "Ajouter à Libris"}</span>
            <h2>{book.id ? "Modifier le livre" : "Nouvelle fiche"}</h2>
          </div>
          <button className="icon-button" onClick={onClose} aria-label="Fermer"><Icon name="close" /></button>
        </header>

        <div className="drawer-scroll">
          <div className="drawer-hero">
            <Cover src={book.coverPath || book.coverUrl} title={book.title || "Livre"} isbn={book.isbn} source={book.source} sourceId={book.sourceId} className="drawer-cover" />
            <div className="drawer-hero-copy">
              <label className="field-label" htmlFor="book-title">Titre</label>
              <input id="book-title" className="title-input" value={book.title} onChange={(e) => patch("title", e.target.value)} placeholder="Titre du livre" />
              <label className="field-label" htmlFor="book-authors">Auteur</label>
              <input id="book-authors" value={book.authors} onChange={(e) => patch("authors", e.target.value)} placeholder="Auteur ou autrice" />
              <div className="compact-grid three">
                <label>
                  <span>Année</span>
                  <input type="number" value={book.publishYear ?? ""} onChange={(e) => patch("publishYear", numberValue(e.target.value))} />
                </label>
                <label>
                  <span>Pages</span>
                  <input type="number" min="0" value={book.pages ?? ""} onChange={(e) => patch("pages", numberValue(e.target.value))} />
                </label>
                <label>
                  <span>ISBN</span>
                  <input value={book.isbn ?? ""} onChange={(e) => patch("isbn", e.target.value || null)} />
                </label>
              </div>
              <div className="edition-fields-grid">
                <label>
                  <span>Éditeur</span>
                  <input value={book.publisher} onChange={(e) => patch("publisher", e.target.value)} placeholder="Belin Gallimard, Folio…" />
                </label>
                <label>
                  <span>Collection</span>
                  <input value={book.collection} onChange={(e) => patch("collection", e.target.value)} placeholder="Classico Lycée, Folio classique…" />
                </label>
                <label>
                  <span>Date de parution</span>
                  <input value={book.publishedDate} onChange={(e) => patch("publishedDate", e.target.value)} placeholder="2015 ou 20 août 2015" />
                </label>
                <label>
                  <span>Format</span>
                  <input value={book.format} onChange={(e) => patch("format", e.target.value)} placeholder="Broché, poche…" />
                </label>
              </div>
              {book.source ? <div className="catalog-origin"><Icon name="check" size={14} /><span>Source : {book.source}</span></div> : null}
            </div>
          </div>

          <section className="form-section">
            <div className="section-heading">
              <div><span className="eyebrow">Lecture</span><h3>Où en êtes-vous ?</h3></div>
            </div>
            <div className="status-grid">
              {statuses.map((status) => (
                <button
                  key={status.value}
                  className={`status-choice ${book.status === status.value ? "is-selected" : ""}`}
                  onClick={() => patch("status", status.value)}
                >
                  <span>{status.label}</span>
                  <small>{status.description}</small>
                  {book.status === status.value ? <Icon name="check" size={17} /> : null}
                </button>
              ))}
            </div>

            {book.status === "reading" ? (
              <div className="progress-editor">
                <div className="progress-editor-top">
                  <label>
                    <span>Page actuelle</span>
                    <input type="number" min="0" max={book.pages ?? undefined} value={book.progressPages} onChange={(e) => patch("progressPages", Math.max(0, Number(e.target.value) || 0))} />
                  </label>
                  <strong>{progress}%</strong>
                </div>
                <div className="progress-track"><span style={{ width: `${progress}%` }} /></div>
              </div>
            ) : null}

            <div className="rating-editor">
              <div>
                <span className="field-label">Votre note</span>
                <p>Une note précise améliore les recommandations.</p>
              </div>
              <div className="rating-buttons" role="radiogroup" aria-label="Note sur 10">
                {Array.from({ length: 10 }, (_, index) => index + 1).map((rating) => (
                  <button
                    key={rating}
                    className={book.rating === rating ? "is-selected" : ""}
                    onClick={() => patch("rating", book.rating === rating ? null : rating)}
                    aria-label={`${rating} sur 10`}
                  >{rating}</button>
                ))}
              </div>
            </div>
          </section>

          <section className="form-section">
            <div className="section-heading">
              <div><span className="eyebrow">Collection physique</span><h3>Votre exemplaire</h3></div>
            </div>
            <label className="toggle-row">
              <input type="checkbox" checked={book.owned} onChange={(e) => patch("owned", e.target.checked)} />
              <span className="toggle-control" />
              <span><strong>Je possède ce livre</strong><small>Il apparaîtra dans votre collection physique.</small></span>
            </label>
            <div className="form-grid two">
              <label><span>Emplacement</span><input value={book.location} onChange={(e) => patch("location", e.target.value)} placeholder="Étagère du salon, bibliothèque…" /></label>
              <label><span>Prêté à</span><input value={book.loanedTo} onChange={(e) => patch("loanedTo", e.target.value)} placeholder="Personne ou vide" /></label>
              <label><span>Prix d’achat</span><input inputMode="decimal" value={book.purchasePrice ?? ""} onChange={(e) => patch("purchasePrice", numberValue(e.target.value))} placeholder="0,00" /></label>
              <label><span>Date d’achat</span><input type="date" value={book.purchaseDate} onChange={(e) => patch("purchaseDate", e.target.value)} /></label>
            </div>
          </section>

          <section className="form-section">
            <div className="section-heading"><div><span className="eyebrow">Mémoire de lecture</span><h3>Ce que vous en retenez</h3></div></div>
            <div className="form-grid two">
              <label><span>Début de lecture</span><input type="date" value={book.startedAt} onChange={(e) => patch("startedAt", e.target.value)} /></label>
              <label><span>Fin de lecture</span><input type="date" value={book.finishedAt} onChange={(e) => patch("finishedAt", e.target.value)} /></label>
            </div>
            <label><span>Ce que vous avez aimé</span><textarea value={book.liked} onChange={(e) => patch("liked", e.target.value)} placeholder="Le rythme, la tension, les personnages, le style…" /></label>
            <label><span>Ce que vous avez moins aimé</span><textarea value={book.disliked} onChange={(e) => patch("disliked", e.target.value)} placeholder="Les longueurs, le ton, certaines parties…" /></label>
            <label><span>Votre avis</span><textarea className="textarea-large" value={book.review} onChange={(e) => patch("review", e.target.value)} placeholder="Votre souvenir du livre, sans obligation de rédiger une critique complète." /></label>
            <label><span>Tags personnels</span><input value={book.tags} onChange={(e) => patch("tags", e.target.value)} placeholder="classique, court, sombre, aventure…" /></label>
          </section>

          <details className="metadata-details">
            <summary>Métadonnées du catalogue <Icon name="chevronDown" size={17} /></summary>
            <div className="metadata-content">
              <label><span>Résumé</span><textarea className="textarea-large" value={book.description} onChange={(e) => patch("description", e.target.value)} /></label>
              <label><span>Thèmes</span><textarea value={book.subjects} onChange={(e) => patch("subjects", e.target.value)} /></label>
              <label><span>URL de couverture</span><input value={book.coverUrl} onChange={(e) => patch("coverUrl", e.target.value)} /></label>
            </div>
          </details>
        </div>

        <footer className="drawer-footer">
          <div>{book.id ? <button className="button button-danger-ghost" onClick={remove} disabled={deleting}><Icon name="trash" size={17} />{deleting ? "Suppression…" : "Supprimer"}</button> : null}</div>
          <div className="drawer-footer-actions">
            <button className="button button-ghost" onClick={onClose}>Annuler</button>
            <button className="button button-primary" onClick={save} disabled={saving || !book.title.trim()}><Icon name="check" size={17} />{saving ? "Enregistrement…" : "Enregistrer"}</button>
          </div>
        </footer>
      </section>
    </div>
  );
}
