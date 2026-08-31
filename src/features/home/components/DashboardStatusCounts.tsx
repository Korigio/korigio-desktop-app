import { Link } from "react-router";
import {
  HOME_PIPELINE_STATUSES,
  HOME_PIPELINE_STATUS_ACCENT_CLASS,
} from "@/features/home/constants";
import type { StatusCount } from "@/features/home/types/dashboard";
import { statusCountMap } from "@/features/home/utils/statusCounts";
import { useI18n } from "@/shared/hooks/useI18n";
import { cn } from "@/shared/utils/cn";

type Props = {
  counts: StatusCount[];
};

export function DashboardStatusCounts({ counts }: Props) {
  const { t } = useI18n();
  const byStatus = statusCountMap(counts);

  return (
    <section className="flex min-w-0 flex-col gap-2">
      <div className="flex flex-wrap gap-2">
        {HOME_PIPELINE_STATUSES.map((status) => {
          const count = byStatus.get(status) ?? 0;
          return (
            <Link
              key={status}
              to={`/repairs?status=${status}`}
              className={cn(
                "min-w-20 rounded-lg border border-border border-l-4 bg-surface px-2.5 py-1.5 text-left",
                HOME_PIPELINE_STATUS_ACCENT_CLASS[status],
                "hover:border-primary focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-primary",
              )}
            >
              <p className="text-lg font-semibold tabular-nums">{count}</p>
              <p className="truncate text-xs text-muted">
                {t(`repairs.status.${status}`)}
              </p>
            </Link>
          );
        })}
      </div>
    </section>
  );
}
