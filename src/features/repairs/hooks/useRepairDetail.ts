import { useEffect, useState } from "react";
import { repairsApi } from "@/features/repairs/api/repairsApi";
import type { Repair } from "@/features/repairs/types/repair";

export function useRepairDetail(repairId: number) {
  const [repair, setRepair] = useState<Repair | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    setLoading(true);
    setError(null);
    void repairsApi
      .get(repairId)
      .then((next) => {
        if (!cancelled) {
          setRepair(next);
        }
      })
      .catch((err: unknown) => {
        if (!cancelled) {
          setRepair(null);
          setError(err instanceof Error ? err.message : "Failed to load repair");
        }
      })
      .finally(() => {
        if (!cancelled) {
          setLoading(false);
        }
      });
    return () => {
      cancelled = true;
    };
  }, [repairId]);

  return { repair, loading, error, setRepair };
}
