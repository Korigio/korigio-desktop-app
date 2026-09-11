import { useCallback, useEffect, useRef, useState } from "react";
import { repairsApi } from "@/features/repairs/api/repairsApi";
import type { Repair } from "@/features/repairs/types/repair";
import { useSyncApplied } from "@/shared/hooks/useSyncApplied";

export function useRepairDetail(repairId: string) {
  const [repair, setRepair] = useState<Repair | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const repairIdRef = useRef(repairId);
  repairIdRef.current = repairId;

  const load = useCallback(
    async (silent: boolean) => {
      const id = repairId;
      if (!silent) {
        setLoading(true);
        setError(null);
      }
      try {
        const next = await repairsApi.get(id);
        if (repairIdRef.current !== id) {
          return;
        }
        setRepair(next);
        setError(null);
      } catch (err) {
        if (repairIdRef.current !== id) {
          return;
        }
        if (!silent) {
          setRepair(null);
          setError(
            err instanceof Error ? err.message : "Failed to load repair",
          );
        }
      } finally {
        if (repairIdRef.current === id && !silent) {
          setLoading(false);
        }
      }
    },
    [repairId],
  );

  useEffect(() => {
    void load(false);
  }, [load]);

  useSyncApplied(() => {
    void load(true);
  });

  return { repair, loading, error, setRepair };
}
