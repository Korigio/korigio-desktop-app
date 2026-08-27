import { useCallback, useEffect, useState } from "react";
import { printApi } from "@/features/print/api/printApi";
import type { RepairPrintReport } from "@/features/print/types/printReport";
import { useI18n } from "@/shared/hooks/useI18n";

export function useRepairPrintReport(repairId: number) {
  const { t } = useI18n();
  const [report, setReport] = useState<RepairPrintReport | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const reload = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const next = await printApi.getRepairReport(repairId);
      setReport(next);
    } catch (err) {
      setReport(null);
      setError(
        err instanceof Error ? err.message : t("print.errors.loadFailed"),
      );
    } finally {
      setLoading(false);
    }
  }, [repairId, t]);

  useEffect(() => {
    void reload();
  }, [reload]);

  return { report, loading, error };
}
