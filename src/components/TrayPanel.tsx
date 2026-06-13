import type { UsageSummary } from "../types/usage";
import { ProgressBar } from "./ProgressBar";
import { ModelBreakdown } from "./ModelBreakdown";

interface TrayPanelProps {
  data: UsageSummary | null;
  loading: boolean;
  error: string | null;
  onRefresh: () => void;
}

function formatTokens(n: number): string {
  if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(2)}M`;
  if (n >= 1_000) return `${(n / 1_000).toFixed(1)}k`;
  return String(n);
}

function formatTime(minutes: number | null): string {
  if (minutes === null) return "—";
  if (minutes <= 0) return "now";
  const h = Math.floor(minutes / 60);
  const m = minutes % 60;
  if (h > 0) return `${h}h ${m}m`;
  return `${m}m`;
}

function formatTimestamp(iso: string): string {
  try {
    return new Date(iso).toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" });
  } catch {
    return "—";
  }
}

interface CardProps {
  children: React.ReactNode;
}

function Card({ children }: CardProps) {
  return (
    <div className="bg-panel-card rounded-2xl px-4 py-3.5 shadow-card">
      {children}
    </div>
  );
}

export function TrayPanel({ data, loading, error, onRefresh }: TrayPanelProps) {
  if (error) {
    return (
      <Card>
        <p className="text-[13px] text-neutral-500 text-center">Could not load usage data.</p>
        <button
          onClick={onRefresh}
          className="mt-2 w-full text-[12px] text-neutral-400 hover:text-neutral-900 transition-colors text-center"
        >
          Try again
        </button>
      </Card>
    );
  }

  if (loading && !data) {
    return (
      <div className="flex flex-col items-center justify-center py-10 gap-2">
        <div className="w-5 h-5 border-2 border-neutral-300 border-t-neutral-800 rounded-full animate-spin" />
        <p className="text-[11px] text-neutral-400">Scanning…</p>
      </div>
    );
  }

  if (!data) return null;

  const { session, weekly, model_breakdown, last_updated } = data;

  return (
    <div className="flex flex-col gap-2">
      {/* Session */}
      <Card>
        <div className="flex items-start justify-between mb-0.5">
          <span className="text-[10px] font-semibold text-neutral-400 uppercase tracking-widest">Session</span>
          {session.minutes_until_reset !== null && (
            <span className="text-[10px] text-neutral-400">
              resets in {formatTime(session.minutes_until_reset)}
            </span>
          )}
        </div>
        <div className="flex items-baseline justify-between mt-1 mb-2.5">
          <span className="text-[26px] font-bold tracking-tight text-neutral-900 tabular-nums leading-none">
            {formatTokens(session.total_tokens)}
          </span>
          <div className="text-right">
            {session.percentage !== null ? (
              <span className="text-[18px] font-semibold text-neutral-900 tabular-nums leading-none">
                {session.percentage.toFixed(0)}%
              </span>
            ) : (
              <span className="text-[11px] text-neutral-400">no limit</span>
            )}
            {session.limit && (
              <p className="text-[10px] text-neutral-400 mt-0.5">of {formatTokens(session.limit)}</p>
            )}
          </div>
        </div>
        <ProgressBar percentage={session.percentage} />
        <div className="mt-2.5 grid grid-cols-2 gap-y-0.5 text-[11px] text-neutral-400">
          <span>↓ {formatTokens(session.input_tokens)} in</span>
          <span>↑ {formatTokens(session.output_tokens)} out</span>
          <span>✦ {formatTokens(session.cache_creation_tokens)} cached</span>
          <span>⚡ {formatTokens(session.cache_read_tokens)} read</span>
        </div>
      </Card>

      {/* Weekly */}
      <Card>
        <div className="flex items-start justify-between mb-0.5">
          <span className="text-[10px] font-semibold text-neutral-400 uppercase tracking-widest">This Week</span>
          {weekly.week_end && (
            <span className="text-[10px] text-neutral-400">
              resets {new Date(weekly.week_end).toLocaleDateString([], { weekday: "short" })}
            </span>
          )}
        </div>
        <div className="flex items-baseline justify-between mt-1 mb-2.5">
          <span className="text-[22px] font-bold tracking-tight text-neutral-900 tabular-nums leading-none">
            {formatTokens(weekly.total_tokens)}
          </span>
          <div className="text-right">
            {weekly.percentage !== null ? (
              <span className="text-[16px] font-semibold text-neutral-900 tabular-nums leading-none">
                {weekly.percentage.toFixed(0)}%
              </span>
            ) : (
              <span className="text-[11px] text-neutral-400">no limit</span>
            )}
            {weekly.limit && (
              <p className="text-[10px] text-neutral-400 mt-0.5">of {formatTokens(weekly.limit)}</p>
            )}
          </div>
        </div>
        <ProgressBar percentage={weekly.percentage} />
        <div className="mt-2.5 grid grid-cols-2 gap-y-0.5 text-[11px] text-neutral-400">
          <span>↓ {formatTokens(weekly.input_tokens)} in</span>
          <span>↑ {formatTokens(weekly.output_tokens)} out</span>
          <span>✦ {formatTokens(weekly.cache_creation_tokens)} cached</span>
          <span>⚡ {formatTokens(weekly.cache_read_tokens)} read</span>
        </div>
      </Card>

      {/* Models */}
      <Card>
        <span className="text-[10px] font-semibold text-neutral-400 uppercase tracking-widest">
          Models
        </span>
        <div className="mt-2.5">
          <ModelBreakdown models={model_breakdown} />
        </div>
      </Card>

      {/* Footer */}
      <div className="flex justify-between items-center px-1 pb-1">
        <span className="text-[11px] text-neutral-400">
          {formatTimestamp(last_updated)}
        </span>
        <button
          onClick={onRefresh}
          disabled={loading}
          className="text-[11px] text-neutral-400 hover:text-neutral-900 disabled:opacity-30 transition-colors flex items-center gap-1"
        >
          <svg
            className={`w-3 h-3 ${loading ? "animate-spin" : ""}`}
            fill="none"
            viewBox="0 0 24 24"
            stroke="currentColor"
            strokeWidth={2.2}
          >
            <path strokeLinecap="round" strokeLinejoin="round"
              d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"
            />
          </svg>
          Refresh
        </button>
      </div>
    </div>
  );
}
