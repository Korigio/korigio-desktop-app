import { HOME_PIPELINE_STATUSES } from "@/features/home/constants";
import type { StatusCount } from "@/features/home/types/dashboard";

export function statusCountMap(counts: StatusCount[] | null | undefined): Map<string, number> {
  return new Map((counts ?? []).map((item) => [item.status, item.count]));
}

export function countForStatus(counts: StatusCount[], status: string): number {
  return statusCountMap(counts).get(status) ?? 0;
}

export function openPipelineTotal(counts: StatusCount[]): number {
  const byStatus = statusCountMap(counts);
  return HOME_PIPELINE_STATUSES.reduce(
    (sum, status) => sum + (byStatus.get(status) ?? 0),
    0,
  );
}
