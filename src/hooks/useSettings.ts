import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { Settings } from "../types/usage";

const DEFAULT_SETTINGS: Settings = {
  plan: "none",
  refresh_interval_secs: 60,
  launch_at_login: false,
  custom_session_limit: null,
  custom_weekly_limit: null,
};

export function useSettings() {
  const [settings, setSettings] = useState<Settings>(DEFAULT_SETTINGS);
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    invoke<Settings>("get_settings")
      .then(setSettings)
      .catch(() => {});
  }, []);

  const save = useCallback(async (updated: Settings) => {
    setSaving(true);
    try {
      await invoke("save_settings", { settings: updated });
      setSettings(updated);
    } finally {
      setSaving(false);
    }
  }, []);

  return { settings, save, saving };
}
