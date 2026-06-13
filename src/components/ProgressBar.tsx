interface ProgressBarProps {
  percentage: number | null;
  className?: string;
}

export function ProgressBar({ percentage, className = "" }: ProgressBarProps) {
  const pct = Math.min(100, Math.max(0, percentage ?? 0));

  return (
    <div className={`h-2 w-full rounded-full bg-panel-muted overflow-hidden ${className}`}>
      <div
        className="h-full rounded-full bg-neutral-800 transition-all duration-500"
        style={{ width: `${pct}%` }}
      />
    </div>
  );
}
