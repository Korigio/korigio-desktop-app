import { useCallback, useEffect, useState } from "react";
import { dashboardApi } from "@/features/home/api/dashboardApi";
import type { HomeDashboard } from "@/features/home/types/dashboard";
import { useI18n } from "@/shared/hooks/useI18n";

export function useHomeDashboard() {
  const { t } = useI18n();
  const [dashboard, setDashboard] = useState<HomeDashboard | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const reload = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const next = await dashboardApi.get();
      setDashboard(next);
    } catch (err) {
      setDashboard(null);
      setError(
        err instanceof Error ? err.message : t("home.dashboard.loadFailed"),
      );
    } finally {
      setLoading(false);
    }
  }, [t]);

  useEffect(() => {
    void reload();
  }, [reload]);

  return { dashboard, loading, error, reload };
}
