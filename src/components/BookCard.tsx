import type { Book, SearchResult } from "../types";
import { Cover } from "./Cover";
import { Icon } from "./Icon";

const statusLabels: Record<string, string> = {
  wishlist: "À lire",
  reading: "En cours",
  read: "Lu",
  abandoned: "Abandonné"
};

export function BookCard({ book, onOpen }: { book: Book; onOpen: (book: Book) => void }) {
  const progress = book.pages && book.progressPages > 0 ? Math.min(100, Math.round((book.progressPages / book.pages) * 100)) : 0;
  return (
    <button className="book-card" onClick={() => onOpen(book)}>
      <div className="book-card-cover-wrap">
        <Cover src={book.coverPath || book.coverUrl} title={book.title} isbn={book.isbn} source={book.source} sourceId={book.sourceId} />
        <span className={`status-pill status-${book.status}`}>{statusLabels[book.status] ?? book.status}</span>
      </div>
      <div className="book-card-body">
        <div>
          <h3>{book.title}</h3>
          <p className="book-author">{book.authors}</p>
          {book.publisher ? <p className="book-edition-line">{book.publisher}{book.collection ? ` · ${book.collection}` : ""}</p> : null}
        </div>
        <div className="book-meta-row">
          <span>{book.publishYear || "Année inconnue"}</span>
          {book.pages ? <span>{book.pages} pages</span> : null}
        </div>
        {book.rating !== null ? (
          <div className="rating-line"><Icon name="star" size={15} /><strong>{book.rating}/10</strong></div>
        ) : (
          <div className="rating-line is-muted">Non noté</div>
        )}
        {book.status === "reading" && book.pages ? (
          <div className="progress-mini" aria-label={`${progress}% lu`}>
            <span style={{ width: `${progress}%` }} />
          </div>
        ) : null}
      </div>
    </button>
  );
}

function editionLine(book: SearchResult) {
  return [book.publisher, book.collection].filter(Boolean).join(" · ");
}

export function SearchBookCard({ book, onAdd }: { book: SearchResult; onAdd: (book: SearchResult) => void }) {
  const edition = editionLine(book);
  return (
    <article className="search-book-card">
      <div className="search-cover-wrap">
        <Cover src={book.coverUrl} title={book.title} isbn={book.isbn} source={book.source} sourceId={book.sourceId} />
        <span className={`catalog-source source-${book.source.toLowerCase().replaceAll(" ", "-")}`}>{book.source || "Catalogue"}</span>
      </div>
      <div className="search-book-card-body">
        <div>
          <div className="eyebrow">{book.publishedDate || book.publishYear || "Édition non datée"}</div>
          <h3>{book.title}</h3>
          <p className="search-author">{book.authors}</p>
          {edition ? <p className="search-edition">{edition}</p> : null}
        </div>

        {book.matchReasons.length ? (
          <div className="match-reasons">
            {book.matchReasons.map((reason) => <span key={reason}><Icon name="check" size={12} />{reason}</span>)}
          </div>
        ) : null}

        <div className="search-book-meta">
          {book.isbn ? <span>ISBN {book.isbn}</span> : <span>ISBN inconnu</span>}
          {book.pages ? <span>{book.pages} pages</span> : <span>Pagination inconnue</span>}
          {book.language ? <span>{book.language.toUpperCase()}</span> : null}
        </div>
        <button className="button button-secondary button-full" onClick={() => onAdd(book)}>
          <Icon name="plus" size={17} /> Ajouter cette édition
        </button>
      </div>
    </article>
  );
}
