import { useEffect, useState } from "react";
import { open, save } from "@tauri-apps/plugin-dialog";
import { api } from "../api";
import { Icon } from "../components/Icon";

export function DataPage({ onImported, notify }: { onImported: () => void; notify: (message: string, kind?: "success" | "error") => void }) {
  const [databasePath, setDatabasePath] = useState("Chargement…");
  useEffect(() => { api.databasePath().then(setDatabasePath).catch(() => setDatabasePath("Indisponible")); }, []);

  async function exportLibrary() {
    const path = await save({ title: "Exporter la bibliothèque Libris", defaultPath: "libris-export.json", filters: [{ name: "Export Libris", extensions: ["json"] }] });
    if (!path) return;
    try { const count = await api.exportData(path); notify(`${count} livre${count > 1 ? "s" : ""} exporté${count > 1 ? "s" : ""}.`); }
    catch (reason) { notify(String(reason), "error"); }
  }

  async function importLibrary() {
    const path = await open({ title: "Importer une bibliothèque Libris", multiple: false, filters: [{ name: "Export Libris", extensions: ["json"] }] });
    if (!path || Array.isArray(path)) return;
    try { const count = await api.importData(path); onImported(); notify(`${count} livre${count > 1 ? "s" : ""} importé${count > 1 ? "s" : ""}.`); }
    catch (reason) { notify(String(reason), "error"); }
  }

  return (
    <div className="page data-page">
      <header className="page-header"><div><span className="eyebrow">Sauvegarde et confidentialité</span><h1>Données</h1><p>Votre bibliothèque est enregistrée localement dans une base SQLite. Aucun compte n’est nécessaire.</p></div></header>
      <section className="data-hero"><div className="data-hero-icon"><Icon name="database" size={34} /></div><div><span className="eyebrow light">Base locale active</span><h2>Vos lectures restent sur cet ordinateur.</h2><p>Les catalogues bibliographiques sont contactés uniquement lors d’une recherche. Vos notes, avis et informations d’achat ne sont pas envoyés.</p></div></section>
      <section className="settings-list">
        <article className="setting-row"><div><span className="setting-icon"><Icon name="database" size={20} /></span><div><h3>Emplacement de la base</h3><p className="path-copy">{databasePath}</p></div></div><span className="status-chip success"><span />Active</span></article>
        <article className="setting-row"><div><span className="setting-icon"><Icon name="download" size={20} /></span><div><h3>Exporter la bibliothèque</h3><p>Crée une sauvegarde JSON lisible contenant toutes vos fiches.</p></div></div><button className="button button-secondary" onClick={() => void exportLibrary()}><Icon name="download" size={17} />Exporter</button></article>
        <article className="setting-row"><div><span className="setting-icon"><Icon name="upload" size={20} /></span><div><h3>Importer une sauvegarde</h3><p>Ajoute les livres d’un export Libris sans supprimer la bibliothèque actuelle.</p></div></div><button className="button button-secondary" onClick={() => void importLibrary()}><Icon name="upload" size={17} />Importer</button></article>
      </section>
    </div>
  );
}
