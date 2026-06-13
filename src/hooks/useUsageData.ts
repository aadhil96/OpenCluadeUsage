import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { UsageSummary } from "../types/usage";

export function useUsageData(refreshIntervalSecs: number) {
  const [data, setData] = useState<UsageSummary | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    try {
      setLoading(true);
      setError(null);
      const summary = await invoke<UsageSummary>("get_usage");
      setData(summary);
    } catch (err) {
      setError(String(err));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    refresh();
    const interval = setInterval(refresh, refreshIntervalSecs * 1000);
    return () => clearInterval(interval);
  }, [refresh, refreshIntervalSecs]);

  // Re-fetch every time the panel window comes into focus (i.e. user opens the tray).
  useEffect(() => {
    window.addEventListener("focus", refresh);
    return () => window.removeEventListener("focus", refresh);
  }, [refresh]);

  return { data, loading, error, refresh };
}
