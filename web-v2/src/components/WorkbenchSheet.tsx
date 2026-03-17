import { X } from 'lucide-react';

interface WorkbenchSheetProps {
  title: string;
  subtitle?: string;
  open: boolean;
  onClose: () => void;
  actions?: React.ReactNode;
  children: React.ReactNode;
}

export function WorkbenchSheet({
  title,
  subtitle,
  open,
  onClose,
  actions,
  children,
}: WorkbenchSheetProps) {
  if (!open) {
    return null;
  }

  return (
    <div className="sheet-backdrop" role="presentation" onClick={onClose}>
      <section
        className="sheet"
        role="dialog"
        aria-modal="true"
        aria-label={title}
        onClick={(event) => event.stopPropagation()}
      >
        <header className="sheet__header">
          <div>
            <p className="workspace-eyebrow">WORKBENCH / ACTIVE</p>
            <h2>{title}</h2>
            {subtitle ? <p>{subtitle}</p> : null}
          </div>
          <div className="sheet__actions">
            {actions}
            <button type="button" className="button button--ghost" onClick={onClose} aria-label="Close workbench">
              <X size={16} />
            </button>
          </div>
        </header>
        <div className="sheet__body">{children}</div>
      </section>
    </div>
  );
}
