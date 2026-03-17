interface KpiCardProps {
  label: string;
  value: string;
  delta: string;
}

export function KpiCard({ label, value, delta }: KpiCardProps) {
  return (
    <article className="kpi-card">
      <span className="kpi-card__label">{label}</span>
      <strong className="kpi-card__value">{value}</strong>
      <span className="kpi-card__delta">{delta}</span>
    </article>
  );
}
