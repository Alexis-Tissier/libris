import { Icon } from "./Icon";

export function Toast({ message, kind = "success", onClose }: { message: string; kind?: "success" | "error"; onClose: () => void }) {
  return (
    <div className={`toast toast-${kind}`} role="status">
      <span className="toast-icon"><Icon name={kind === "success" ? "check" : "x"} size={17} /></span>
      <span>{message}</span>
      <button onClick={onClose} aria-label="Fermer"><Icon name="close" size={16} /></button>
    </div>
  );
}
