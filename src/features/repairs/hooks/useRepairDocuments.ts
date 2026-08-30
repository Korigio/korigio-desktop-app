import { useCallback, useEffect, useState } from "react";
import { repairsApi } from "@/features/repairs/api/repairsApi";
import type { RepairDocument } from "@/features/repairs/types/repairDocument";
import { useSyncApplied } from "@/shared/hooks/useSyncApplied";

export function useRepairDocuments(repairId: string) {
  const [documents, setDocuments] = useState<RepairDocument[]>([]);
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
        const next = await repairsApi.listDocuments(repairId);
        setDocuments(next);
        setError(null);
      } catch (err) {
        if (!silent) {
          setDocuments([]);
          setError(
            err instanceof Error ? err.message : "Failed to load documents",
          );
        }
      } finally {
        if (!silent) {
          setLoading(false);
        }
      }
    },
    [repairId],
  );

  useEffect(() => {
    void reload();
  }, [reload]);

  useSyncApplied(() => {
    void reload({ silent: true });
  });

  return { documents, loading, error, reload, setDocuments };
}
