import type { ModelUsage } from "../types/usage";

interface ModelBreakdownProps {
  models: ModelUsage[];
}

function formatTokens(n: number): string {
  if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}M`;
  if (n >= 1_000) return `${(n / 1_000).toFixed(1)}k`;
  return String(n);
}

export function ModelBreakdown({ models }: ModelBreakdownProps) {
  if (models.length === 0) {
    return <p className="text-[11px] text-neutral-400 italic">No model data for this session.</p>;
  }

  const maxTokens = Math.max(...models.map((m) => m.total_tokens), 1);

  return (
    <div className="space-y-2.5">
      {models.map((model) => (
        <div key={model.model}>
          <div className="flex justify-between items-baseline mb-1">
            <span className="text-[12px] font-medium text-neutral-800">{model.display_name}</span>
            <span className="text-[11px] text-neutral-400 tabular-nums">
              {formatTokens(model.total_tokens)}
            </span>
          </div>
          <div className="h-1.5 w-full rounded-full bg-panel-muted overflow-hidden">
            <div
              className="h-full rounded-full bg-neutral-800 transition-all duration-500"
              style={{ width: `${(model.total_tokens / maxTokens) * 100}%` }}
            />
          </div>
        </div>
      ))}
    </div>
  );
}
