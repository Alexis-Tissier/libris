import { useEffect, useMemo, useRef, useState } from "react";
import { convertFileSrc } from "@tauri-apps/api/core";
import { api } from "../api";
import { Icon } from "./Icon";

type CoverProps = {
  src?: string;
  title: string;
  isbn?: string | null;
  source?: string;
  sourceId?: string;
  className?: string;
};

const resolvedCoverCache = new Map<string, string | null>();

function cacheKey(src: string, isbn: string | null, source: string, sourceId: string) {
  return [src, isbn ?? "", source, sourceId].join("|");
}

export function Cover({
  src = "",
  title,
  isbn = null,
  source = "",
  sourceId = "",
  className = ""
}: CoverProps) {
  const hostRef = useRef<HTMLDivElement>(null);
  const localSource = useMemo(() => {
    if (!src) return "";
    if (src.startsWith("/") || /^[A-Za-z]:\\/.test(src)) {
      try { return convertFileSrc(src); } catch { return src; }
    }
    return "";
  }, [src]);
  const key = useMemo(() => cacheKey(src, isbn, source, sourceId), [src, isbn, source, sourceId]);
  const [resolvedSrc, setResolvedSrc] = useState(localSource);
  const [loading, setLoading] = useState(!localSource && Boolean(src || isbn || sourceId));
  const [failed, setFailed] = useState(false);
  const [visible, setVisible] = useState(false);

  useEffect(() => {
    setResolvedSrc(localSource);
    setFailed(false);
    setLoading(!localSource && Boolean(src || isbn || sourceId));
  }, [key, localSource, src, isbn, sourceId]);

  useEffect(() => {
    const node = hostRef.current;
    if (!node || localSource) {
      setVisible(true);
      return;
    }
    const observer = new IntersectionObserver(
      ([entry]) => {
        if (entry.isIntersecting) {
          setVisible(true);
          observer.disconnect();
        }
      },
      { rootMargin: "320px" }
    );
    observer.observe(node);
    return () => observer.disconnect();
  }, [key, localSource]);

  useEffect(() => {
    if (!visible || localSource || (!src && !isbn && !sourceId)) return;
    let cancelled = false;
    const cached = resolvedCoverCache.get(key);
    if (cached !== undefined) {
      setResolvedSrc(cached ?? "");
      setLoading(false);
      setFailed(!cached);
      return;
    }

    setLoading(true);
    void api.resolveCover(src, isbn, source, sourceId)
      .then((value) => {
        if (cancelled) return;
        resolvedCoverCache.set(key, value);
        setResolvedSrc(value ?? "");
        setFailed(!value);
      })
      .catch(() => {
        if (cancelled) return;
        // Le mode aperçu dans un navigateur n’a pas accès aux commandes Tauri.
        // Une URL distante reste alors utilisable directement.
        const fallback = /^https?:\/\//.test(src) ? src : "";
        resolvedCoverCache.set(key, fallback || null);
        setResolvedSrc(fallback);
        setFailed(!fallback);
      })
      .finally(() => {
        if (!cancelled) setLoading(false);
      });

    return () => { cancelled = true; };
  }, [visible, localSource, src, isbn, source, sourceId, key]);

  return (
    <div ref={hostRef} className="cover-host">
      {resolvedSrc && !failed ? (
        <img
          className={`book-cover ${className}`}
          src={resolvedSrc}
          alt={`Couverture de ${title}`}
          loading="lazy"
          decoding="async"
          onError={() => setFailed(true)}
        />
      ) : (
        <div className={`cover-placeholder ${className}${loading ? " is-loading" : ""}`} aria-label={`Pas de couverture pour ${title}`}>
          <Icon name="book" size={30} />
          <span>{title.slice(0, 1).toUpperCase()}</span>
          {loading ? <small>Couverture…</small> : null}
        </div>
      )}
    </div>
  );
}
