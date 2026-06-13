import { useState } from "react";
import type { Settings, Plan } from "../types/usage";
import { PLAN_LABELS } from "../types/usage";

interface SettingsPanelProps {
  settings: Settings;
  onSave: (s: Settings) => void;
  saving: boolean;
}

const REFRESH_OPTIONS = [
  { label: "10s", value: 10 },
  { label: "30s", value: 30 },
  { label: "1m", value: 60 },
  { label: "5m", value: 300 },
];

const PLAN_NOTES: Record<Plan, string> = {
  pro: "≈1.2M / session • ≈8M / week",
  max5x: "≈6M / session • ≈40M / week",
  max20x: "≈24M / session • ≈160M / week",
  none: "Raw counts only, no % bars",
};

function Card({ children }: { children: React.ReactNode }) {
  return (
    <div className="bg-panel-card rounded-2xl px-4 py-3.5 shadow-card">
      {children}
    </div>
  );
}

export function SettingsPanel({ settings, onSave, saving }: SettingsPanelProps) {
  const [local, setLocal] = useState<Settings>(settings);

  return (
    <div className="flex flex-col gap-2">
      {/* Plan */}
      <Card>
        <p className="text-[10px] font-semibold text-neutral-400 uppercase tracking-widest mb-2.5">
          Subscription Plan
        </p>
        <div className="space-y-1">
          {(Object.keys(PLAN_LABELS) as Plan[]).map((plan) => (
            <button
              key={plan}
              onClick={() => setLocal({ ...local, plan })}
              className={`w-full flex items-center justify-between px-3 py-2 rounded-xl transition-colors text-left ${
                local.plan === plan
                  ? "bg-neutral-900 text-white"
                  : "bg-panel-muted text-neutral-700 hover:bg-neutral-200"
              }`}
            >
              <div>
                <p className="text-[13px] font-medium leading-tight">{PLAN_LABELS[plan]}</p>
                <p className={`text-[10px] mt-0.5 ${local.plan === plan ? "text-neutral-400" : "text-neutral-400"}`}>
                  {PLAN_NOTES[plan]}
                </p>
              </div>
              {local.plan === plan && (
                <svg className="w-4 h-4 text-white shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2.5}>
                  <path strokeLinecap="round" strokeLinejoin="round" d="M5 13l4 4L19 7" />
                </svg>
              )}
            </button>
          ))}
        </div>
      </Card>

      {/* Custom limits */}
      {local.plan === "none" && (
        <Card>
          <p className="text-[10px] font-semibold text-neutral-400 uppercase tracking-widest mb-2.5">
            Custom Limits
          </p>
          <div className="flex gap-2">
            <div className="flex-1">
              <label className="text-[11px] text-neutral-400 mb-1 block">Session</label>
              <input
                type="number"
                placeholder="144000"
                value={local.custom_session_limit ?? ""}
                onChange={(e) =>
                  setLocal({ ...local, custom_session_limit: e.target.value ? Number(e.target.value) : null })
                }
                className="w-full bg-panel-muted rounded-xl px-3 py-2 text-[12px] text-neutral-900 focus:outline-none focus:ring-2 focus:ring-neutral-900"
              />
            </div>
            <div className="flex-1">
              <label className="text-[11px] text-neutral-400 mb-1 block">Weekly</label>
              <input
                type="number"
                placeholder="1000000"
                value={local.custom_weekly_limit ?? ""}
                onChange={(e) =>
                  setLocal({ ...local, custom_weekly_limit: e.target.value ? Number(e.target.value) : null })
                }
                className="w-full bg-panel-muted rounded-xl px-3 py-2 text-[12px] text-neutral-900 focus:outline-none focus:ring-2 focus:ring-neutral-900"
              />
            </div>
          </div>
        </Card>
      )}

      {/* Refresh interval */}
      <Card>
        <p className="text-[10px] font-semibold text-neutral-400 uppercase tracking-widest mb-2.5">
          Refresh Interval
        </p>
        <div className="flex gap-1 bg-panel-muted p-1 rounded-xl">
          {REFRESH_OPTIONS.map((opt) => (
            <button
              key={opt.value}
              onClick={() => setLocal({ ...local, refresh_interval_secs: opt.value })}
              className={`flex-1 py-1.5 rounded-lg text-[12px] font-medium transition-all ${
                local.refresh_interval_secs === opt.value
                  ? "bg-white text-neutral-900 shadow-card"
                  : "text-neutral-500 hover:text-neutral-800"
              }`}
            >
              {opt.label}
            </button>
          ))}
        </div>
      </Card>

      {/* Launch at login */}
      <Card>
        <button
          onClick={() => setLocal({ ...local, launch_at_login: !local.launch_at_login })}
          className="w-full flex items-center justify-between"
        >
          <span className="text-[13px] text-neutral-800">Launch at login</span>
          <div className={`w-10 h-6 rounded-full transition-colors relative ${local.launch_at_login ? "bg-neutral-900" : "bg-panel-muted"}`}>
            <div className={`absolute top-1 w-4 h-4 bg-white rounded-full shadow transition-all ${local.launch_at_login ? "left-5" : "left-1"}`} />
          </div>
        </button>
      </Card>

      {/* Save */}
      <button
        onClick={() => onSave(local)}
        disabled={saving}
        className="w-full py-2.5 rounded-2xl bg-neutral-900 hover:bg-neutral-700 disabled:opacity-50 text-white text-[13px] font-semibold transition-colors shadow-card"
      >
        {saving ? "Saving…" : "Save Settings"}
      </button>
    </div>
  );
}
