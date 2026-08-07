export type BookStatus = "wishlist" | "reading" | "read" | "abandoned";

export interface Book {
  id: number | null;
  workKey: string | null;
  editionKey: string | null;
  source: string;
  sourceId: string;
  isbn: string | null;
  title: string;
  authors: string;
  publisher: string;
  collection: string;
  publishedDate: string;
  format: string;
  description: string;
  subjects: string;
  publishYear: number | null;
  pages: number | null;
  language: string;
  coverUrl: string;
  coverPath: string;
  status: BookStatus;
  owned: boolean;
  rating: number | null;
  review: string;
  liked: string;
  disliked: string;
  location: string;
  purchasePrice: number | null;
  purchaseDate: string;
  startedAt: string;
  finishedAt: string;
  progressPages: number;
  loanedTo: string;
  tags: string;
  createdAt: string;
  updatedAt: string;
}

export interface SearchResult {
  workKey: string;
  editionKey: string | null;
  source: string;
  sourceId: string;
  isbn: string | null;
  /** ISBN connus pour la même œuvre ; utilisé uniquement par les recommandations. */
  alternateIsbns: string[];
  title: string;
  authors: string;
  publisher: string;
  collection: string;
  publishedDate: string;
  format: string;
  description: string;
  subjects: string;
  publishYear: number | null;
  pages: number | null;
  language: string;
  coverUrl: string;
  editionCount: number;
  relevanceScore: number;
  matchReasons: string[];
}

export interface NamedCount {
  name: string;
  count: number;
}

export interface DashboardStats {
  total: number;
  owned: number;
  read: number;
  reading: number;
  wishlist: number;
  abandoned: number;
  averageRating: number;
  pagesRead: number;
  profiledBooks: number;
  topAuthors: NamedCount[];
  topSubjects: NamedCount[];
}

export interface Recommendation {
  book: SearchResult;
  score: number;
  reasons: string[];
  warnings: string[];
}

export type PageId = "home" | "library" | "discover" | "profile" | "data";

export function emptyBook(source?: SearchResult): Book {
  return {
    id: null,
    workKey: source?.workKey ?? null,
    editionKey: source?.editionKey ?? null,
    source: source?.source ?? "",
    sourceId: source?.sourceId ?? "",
    isbn: source?.isbn ?? null,
    title: source?.title ?? "",
    authors: source?.authors ?? "",
    publisher: source?.publisher ?? "",
    collection: source?.collection ?? "",
    publishedDate: source?.publishedDate ?? "",
    format: source?.format ?? "",
    description: source?.description ?? "",
    subjects: source?.subjects ?? "",
    publishYear: source?.publishYear ?? null,
    pages: source?.pages ?? null,
    language: source?.language ?? "fr",
    coverUrl: source?.coverUrl ?? "",
    coverPath: "",
    status: "wishlist",
    owned: false,
    rating: null,
    review: "",
    liked: "",
    disliked: "",
    location: "",
    purchasePrice: null,
    purchaseDate: "",
    startedAt: "",
    finishedAt: "",
    progressPages: 0,
    loanedTo: "",
    tags: "",
    createdAt: "",
    updatedAt: ""
  };
}
