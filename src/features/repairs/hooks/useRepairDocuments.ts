import { useCallback, useEffect, useState } from "react";
import { repairsApi } from "@/features/repairs/api/repairsApi";
import type { RepairDocument } from "@/features/repairs/types/repairDocument";

export function useRepairDocuments(repairId: number) {
  const [documents, setDocuments] = useState<RepairDocument[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const reload = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const next = await repairsApi.listDocuments(repairId);
      setDocuments(next);
    } catch (err) {
      setDocuments([]);
      setError(err instanceof Error ? err.message : "Failed to load documents");
    } finally {
      setLoading(false);
    }
  }, [repairId]);

  useEffect(() => {
    void reload();
  }, [reload]);

  return { documents, loading, error, reload, setDocuments };
}
