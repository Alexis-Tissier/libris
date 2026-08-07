import type { ReactNode } from "react";
import type { PageId } from "../types";
import { Icon, type IconName } from "./Icon";

const items: { id: PageId; label: string; icon: IconName }[] = [
  { id: "home", label: "Accueil", icon: "home" },
  { id: "library", label: "Bibliothèque", icon: "library" },
  { id: "discover", label: "Découvrir", icon: "search" },
  { id: "profile", label: "Pour vous", icon: "sparkles" },
  { id: "data", label: "Données", icon: "database" }
];

export function Shell({ page, onNavigate, children }: { page: PageId; onNavigate: (page: PageId) => void; children: ReactNode }) {
  return (
    <div className="app-shell">
      <aside className="sidebar">
        <button className="brand" onClick={() => onNavigate("home")} aria-label="Accueil Libris">
          <span className="brand-mark">L</span>
          <span>
            <strong>Libris</strong>
            <small>Bibliothèque personnelle</small>
          </span>
        </button>

        <nav className="sidebar-nav" aria-label="Navigation principale">
          {items.map((item) => (
            <button
              key={item.id}
              className={`nav-item ${page === item.id ? "is-active" : ""}`}
              onClick={() => onNavigate(item.id)}
            >
              <Icon name={item.icon} size={19} />
              <span>{item.label}</span>
            </button>
          ))}
        </nav>

      </aside>
      <main className="main-area">{children}</main>
    </div>
  );
}
