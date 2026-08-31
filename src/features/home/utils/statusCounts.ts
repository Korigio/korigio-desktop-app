import type { StatusCount } from "@/features/home/types/dashboard";

export function statusCountMap(counts: StatusCount[] | null | undefined): Map<string, number> {
  return new Map((counts ?? []).map((item) => [item.status, item.count]));
}
