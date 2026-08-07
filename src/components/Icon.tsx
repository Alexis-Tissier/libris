import type { ReactElement, SVGProps } from "react";

export type IconName =
  | "home" | "library" | "search" | "sparkles" | "database" | "plus"
  | "arrowRight" | "book" | "close" | "check" | "trash" | "download"
  | "upload" | "refresh" | "chevronDown" | "heart" | "clock" | "mapPin"
  | "user" | "more" | "star" | "filter" | "x" | "external" | "bookmark";

const paths: Record<IconName, ReactElement> = {
  home: <><path d="m3 11 9-8 9 8"/><path d="M5 10v10h14V10"/><path d="M9 20v-6h6v6"/></>,
  library: <><path d="M4 19.5A2.5 2.5 0 0 1 6.5 17H20"/><path d="M6.5 2H20v20H6.5A2.5 2.5 0 0 1 4 19.5v-15A2.5 2.5 0 0 1 6.5 2Z"/></>,
  search: <><circle cx="11" cy="11" r="7"/><path d="m20 20-4-4"/></>,
  sparkles: <><path d="m12 3-1.2 3.6a2 2 0 0 1-1.2 1.2L6 9l3.6 1.2a2 2 0 0 1 1.2 1.2L12 15l1.2-3.6a2 2 0 0 1 1.2-1.2L18 9l-3.6-1.2a2 2 0 0 1-1.2-1.2Z"/><path d="M5 3v4"/><path d="M3 5h4"/><path d="M19 17v4"/><path d="M17 19h4"/></>,
  database: <><ellipse cx="12" cy="5" rx="8" ry="3"/><path d="M4 5v6c0 1.7 3.6 3 8 3s8-1.3 8-3V5"/><path d="M4 11v6c0 1.7 3.6 3 8 3s8-1.3 8-3v-6"/></>,
  plus: <><path d="M12 5v14"/><path d="M5 12h14"/></>,
  arrowRight: <><path d="M5 12h14"/><path d="m13 6 6 6-6 6"/></>,
  book: <><path d="M4 19.5A2.5 2.5 0 0 1 6.5 17H20"/><path d="M6.5 2H20v20H6.5A2.5 2.5 0 0 1 4 19.5v-15A2.5 2.5 0 0 1 6.5 2Z"/><path d="M8 7h8"/><path d="M8 11h5"/></>,
  close: <><path d="m6 6 12 12"/><path d="M18 6 6 18"/></>,
  x: <><path d="m6 6 12 12"/><path d="M18 6 6 18"/></>,
  check: <path d="m5 12 4 4L19 6"/>,
  trash: <><path d="M3 6h18"/><path d="M8 6V4h8v2"/><path d="m19 6-1 14H6L5 6"/><path d="M10 11v5"/><path d="M14 11v5"/></>,
  download: <><path d="M12 3v12"/><path d="m7 10 5 5 5-5"/><path d="M5 21h14"/></>,
  upload: <><path d="M12 21V9"/><path d="m7 14 5-5 5 5"/><path d="M5 3h14"/></>,
  refresh: <><path d="M20 6v5h-5"/><path d="M4 18v-5h5"/><path d="M18.5 9A7 7 0 0 0 6 5.5L4 8"/><path d="M5.5 15A7 7 0 0 0 18 18.5L20 16"/></>,
  chevronDown: <path d="m6 9 6 6 6-6"/>,
  heart: <path d="M20.8 4.6a5.5 5.5 0 0 0-7.8 0L12 5.6l-1-1a5.5 5.5 0 0 0-7.8 7.8l1 1L12 21l7.8-7.6 1-1a5.5 5.5 0 0 0 0-7.8Z"/>,
  clock: <><circle cx="12" cy="12" r="9"/><path d="M12 7v5l3 2"/></>,
  mapPin: <><path d="M20 10c0 5-8 11-8 11S4 15 4 10a8 8 0 1 1 16 0Z"/><circle cx="12" cy="10" r="2"/></>,
  user: <><circle cx="12" cy="8" r="4"/><path d="M4 21a8 8 0 0 1 16 0"/></>,
  more: <><circle cx="5" cy="12" r="1"/><circle cx="12" cy="12" r="1"/><circle cx="19" cy="12" r="1"/></>,
  star: <path d="m12 2 3.1 6.3 6.9 1-5 4.9 1.2 6.8-6.2-3.2L5.8 21 7 14.2l-5-4.9 6.9-1Z"/>,
  filter: <><path d="M4 6h16"/><path d="M7 12h10"/><path d="M10 18h4"/></>,
  external: <><path d="M15 3h6v6"/><path d="m10 14 11-11"/><path d="M18 13v7H4V6h7"/></>,
  bookmark: <path d="M6 3h12v18l-6-4-6 4Z"/>
};

export function Icon({ name, size = 20, ...props }: SVGProps<SVGSVGElement> & { name: IconName; size?: number }) {
  return (
    <svg
      width={size}
      height={size}
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.8"
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
      {...props}
    >
      {paths[name]}
    </svg>
  );
}
