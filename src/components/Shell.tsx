import { useEffect, useState, type ReactNode } from "react";
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
  const [mobileOpen, setMobileOpen] = useState(false);

  useEffect(() => setMobileOpen(false), [page]);

  function navigate(next: PageId) {
    setMobileOpen(false);
    onNavigate(next);
  }

  return (
    <div className={`app-shell${mobileOpen ? " mobile-nav-open" : ""}`}>
      <aside className={`sidebar${mobileOpen ? " is-mobile-open" : ""}`}>
        <button className="brand" onClick={() => navigate("home")} aria-label="Accueil Libris">
          <span className="brand-mark">L</span>
          <span>
            <strong>Libris</strong>
            <small>Bibliothèque personnelle</small>
          </span>
        </button>

        <button
          type="button"
          className="mobile-nav-toggle"
          aria-label={mobileOpen ? "Fermer le menu" : "Ouvrir le menu"}
          aria-expanded={mobileOpen}
          onClick={() => setMobileOpen((value) => !value)}
        >
          <span />
          <span />
          <span />
        </button>

        <nav className="sidebar-nav" aria-label="Navigation principale">
          {items.map((item) => (
            <button
              key={item.id}
              className={`nav-item ${page === item.id ? "is-active" : ""}`}
              onClick={() => navigate(item.id)}
            >
              <Icon name={item.icon} size={19} />
              <span>{item.label}</span>
            </button>
          ))}
        </nav>
      </aside>

      {mobileOpen ? <button className="mobile-nav-backdrop" type="button" aria-label="Fermer le menu" onClick={() => setMobileOpen(false)} /> : null}
      <main className="main-area">{children}</main>
    </div>
  );
}
