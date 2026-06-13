import { useState } from "react";
import { TrayPanel } from "./components/TrayPanel";
import { SettingsPanel } from "./components/SettingsPanel";
import { useUsageData } from "./hooks/useUsageData";
import { useSettings } from "./hooks/useSettings";

export default function App() {
  const [view, setView] = useState<"panel" | "settings">("panel");
  const { settings, save, saving } = useSettings();
  const { data, loading, error, refresh } = useUsageData(settings.refresh_interval_secs);

  return (
    <div
      className="h-screen bg-panel-bg text-neutral-900 flex flex-col rounded-2xl overflow-hidden"
      style={{ WebkitUserSelect: "none" }}
    >
      {/* Header */}
      <div className="flex items-center justify-between px-4 pt-4 pb-2">
        <span className="text-[15px] font-semibold tracking-tight text-neutral-900">
          {view === "settings" ? "Settings" : "ClaudeUsage"}
        </span>
        {view === "settings" ? (
          <button
            onClick={() => setView("panel")}
            className="text-[13px] font-medium text-neutral-400 hover:text-neutral-900 transition-colors"
          >
            Done
          </button>
        ) : (
          <button
            onClick={() => setView("settings")}
            className="w-7 h-7 flex items-center justify-center rounded-full bg-panel-card shadow-card text-neutral-400 hover:text-neutral-700 transition-colors"
            title="Settings"
          >
            <svg className="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2.2}>
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z"
              />
              <path strokeLinecap="round" strokeLinejoin="round" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
            </svg>
          </button>
        )}
      </div>

      {/* Body */}
      <div className="flex-1 overflow-y-auto px-3 pb-3 space-y-2">
        {view === "panel" ? (
          <TrayPanel data={data} loading={loading} error={error} onRefresh={refresh} />
        ) : (
          <SettingsPanel settings={settings} onSave={save} saving={saving} />
        )}
      </div>
    </div>
  );
}
