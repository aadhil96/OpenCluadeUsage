import { useState, useEffect, useCallback, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { UsageSummary } from "../types/usage";

const MIN_INTERVAL_SECS = 5;

export function useUsageData(refreshIntervalSecs: number) {
  const [data, setData] = useState<UsageSummary | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const inFlight = useRef(false);

  const refresh = useCallback(async () => {
    if (inFlight.current) return;
    inFlight.current = true;
    try {
      setLoading(true);
      setError(null);
      const summary = await invoke<UsageSummary>("get_usage");
      setData(summary);
    } catch (err) {
      setError(String(err));
    } finally {
      setLoading(false);
      inFlight.current = false;
    }
  }, []);

  // Initial fetch + safety-net interval. The backend poller emits
  // "usage-updated" so this interval mostly just covers the case where the
  // event arrives before this hook mounts; clamp to MIN_INTERVAL_SECS to
  // prevent settings-corruption from causing a tight loop.
  useEffect(() => {
    refresh();
    const secs = Math.max(
      MIN_INTERVAL_SECS,
      Number.isFinite(refreshIntervalSecs) ? refreshIntervalSecs : MIN_INTERVAL_SECS,
    );
    const interval = setInterval(refresh, secs * 1000);
    return () => clearInterval(interval);
  }, [refresh, refreshIntervalSecs]);

  // Re-fetch when the panel comes into focus.
  useEffect(() => {
    window.addEventListener("focus", refresh);
    return () => window.removeEventListener("focus", refresh);
  }, [refresh]);

  // Subscribe to backend-pushed updates so the UI reflects tray changes
  // without an extra IPC round-trip.
  useEffect(() => {
    let unlisten: (() => void) | undefined;
    listen<UsageSummary>("usage-updated", (event) => {
      setData(event.payload);
      setError(null);
      setLoading(false);
    }).then((fn) => {
      unlisten = fn;
    });
    return () => {
      unlisten?.();
    };
  }, []);

  return { data, loading, error, refresh };
}
