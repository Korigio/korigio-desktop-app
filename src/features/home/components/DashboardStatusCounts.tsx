import { Link } from "react-router";
import type { StatusCount } from "@/features/home/types/dashboard";
import { useI18n } from "@/shared/hooks/useI18n";
import { cn } from "@/shared/utils/cn";

type Props = {
  counts: StatusCount[];
};

const PIPELINE = [
  "received",
  "diagnosis",
  "waiting_customer",
  "waiting_part",
  "in_repair",
  "ready",
  "awaiting_pickup",
] as const;

export function DashboardStatusCounts({ counts }: Props) {
  const { t } = useI18n();
  const byStatus = new Map(counts.map((item) => [item.status, item.count]));

  return (
    <section className="flex min-w-0 flex-col gap-2">
      <h2 className="text-sm font-semibold">{t("home.dashboard.pipeline")}</h2>
      <div className="grid grid-cols-2 gap-2 sm:grid-cols-3 xl:grid-cols-7">
        {PIPELINE.map((status) => {
          const count = byStatus.get(status) ?? 0;
          return (
            <Link
              key={status}
              to={`/repairs?status=${status}`}
              className={cn(
                "min-w-0 border border-border bg-surface px-3 py-2 text-left",
                "hover:border-primary focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-primary",
              )}
            >
              <p className="text-2xl font-semibold tabular-nums">{count}</p>
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
