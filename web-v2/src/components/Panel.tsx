import type { PropsWithChildren, ReactNode } from 'react';

interface PanelProps extends PropsWithChildren {
  title: string;
  subtitle?: string;
  action?: ReactNode;
  className?: string;
}

export function Panel({ title, subtitle, action, className, children }: PanelProps) {
  return (
    <section className={`panel ${className ?? ''}`.trim()}>
      <header className="panel-header">
        <div>
          <h3>{title}</h3>
          {subtitle ? <p>{subtitle}</p> : null}
        </div>
        {action ? <div className="panel-action">{action}</div> : null}
      </header>
      {children}
    </section>
  );
}
