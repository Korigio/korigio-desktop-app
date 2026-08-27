import { Link } from "react-router";
import type { DashboardRepairRow } from "@/features/home/types/dashboard";
import { useI18n } from "@/shared/hooks/useI18n";

type Props = {
  title: string;
  empty: string;
  rows: DashboardRepairRow[];
  showReadyAt?: boolean;
};

export function DashboardRepairList({
  title,
  empty,
  rows,
  showReadyAt = false,
}: Props) {
  const { t } = useI18n();

  return (
    <section className="min-w-0 flex flex-col gap-2">
      <h2 className="text-sm font-semibold">{title}</h2>
      {rows.length === 0 ? (
        <p className="text-sm text-muted">{empty}</p>
      ) : (
        <ul className="divide-y divide-border border-y border-border">
          {rows.map((row) => (
            <li key={row.id} className="min-w-0 py-2">
              <Link
                to={`/repairs/${row.id}`}
                className="flex min-w-0 flex-col gap-0.5 text-sm hover:text-primary"
              >
                <span className="truncate font-medium">
                  {row.repairNumber} · {row.customerName}
                </span>
                <span className="break-words text-muted">
                  {t(`repairs.status.${row.status}`)}
                  {row.customerPhone ? ` · ${row.customerPhone}` : ""}
                  {` · ${t("home.dashboard.daysInStatus").replace("{days}", String(row.daysInStatus))}`}
                  {showReadyAt && row.readyAt
                    ? ` · ${t("home.dashboard.readyAt").replace("{date}", row.readyAt.slice(0, 10))}`
                    : ""}
                </span>
              </Link>
            </li>
          ))}
        </ul>
      )}
    </section>
  );
}
