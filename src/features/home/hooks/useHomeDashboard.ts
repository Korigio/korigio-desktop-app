import { useCallback, useEffect, useState } from "react";
import { dashboardApi } from "@/features/home/api/dashboardApi";
import type { HomeDashboard } from "@/features/home/types/dashboard";
import { useI18n } from "@/shared/hooks/useI18n";
import { useSyncApplied } from "@/shared/hooks/useSyncApplied";

export function useHomeDashboard() {
  const { t } = useI18n();
  const [dashboard, setDashboard] = useState<HomeDashboard | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const reload = useCallback(
    async (reloadOptions?: { silent?: boolean }) => {
      const silent = reloadOptions?.silent === true;
      if (!silent) {
        setLoading(true);
        setError(null);
      }
      try {
        const next = await dashboardApi.get();
        setDashboard(next);
        setError(null);
      } catch (err) {
        if (!silent) {
          setDashboard(null);
          setError(
            err instanceof Error ? err.message : t("home.dashboard.loadFailed"),
          );
        }
      } finally {
        if (!silent) {
          setLoading(false);
        }
      }
    },
    [t],
  );

  useEffect(() => {
    void reload();
  }, [reload]);

  useSyncApplied(() => {
    void reload({ silent: true });
  });

  return { dashboard, loading, error, reload };
}
