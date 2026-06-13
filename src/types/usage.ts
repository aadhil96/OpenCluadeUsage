export interface TokenUsage {
  input_tokens: number;
  output_tokens: number;
  cache_creation_tokens: number;
  cache_read_tokens: number;
  total_tokens: number;
}

export interface SessionUsage extends TokenUsage {
  session_start: string | null;
  session_end: string | null;
  minutes_until_reset: number | null;
  limit: number | null;
  percentage: number | null;
}

export interface WeeklyUsage extends TokenUsage {
  week_start: string;
  week_end: string;
  limit: number | null;
  percentage: number | null;
}

export interface ModelUsage {
  model: string;
  display_name: string;
  model_family: string;
  input_tokens: number;
  output_tokens: number;
  cache_creation_tokens: number;
  cache_read_tokens: number;
  total_tokens: number;
  cost_usd: number;
}

export interface CostEstimate {
  session_cost_usd: number;
  weekly_cost_usd: number;
}

export interface UsageSummary {
  session: SessionUsage;
  weekly: WeeklyUsage;
  model_breakdown: ModelUsage[];
  cost_estimate: CostEstimate;
  last_updated: string;
  plan: string | null;
}

export type Plan = "pro" | "max5x" | "max20x" | "none";

export interface Settings {
  plan: Plan;
  refresh_interval_secs: number;
  launch_at_login: boolean;
  custom_session_limit: number | null;
  custom_weekly_limit: number | null;
}

export const PLAN_LABELS: Record<Plan, string> = {
  pro: "Claude Pro",
  max5x: "Claude Max (5×)",
  max20x: "Claude Max (20×)",
  none: "No limit tracking",
};
