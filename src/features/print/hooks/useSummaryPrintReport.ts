import { useCallback, useEffect, useState } from "react";
import { printApi } from "@/features/print/api/printApi";
import type { SummaryPrintReport } from "@/features/print/types/printReport";

export function useSummaryPrintReport(repairId: string) {
  const [report, setReport] = useState<SummaryPrintReport | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const reload = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const next = await printApi.getSummaryReport(repairId);
      setReport(next);
    } catch (err) {
      setReport(null);
      setError(
        err instanceof Error ? err.message : "Could not load summary report",
      );
    } finally {
      setLoading(false);
    }
  }, [repairId]);

  useEffect(() => {
    void reload();
  }, [reload]);

  return { report, loading, error };
}
