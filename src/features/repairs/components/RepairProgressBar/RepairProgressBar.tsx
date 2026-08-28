import {
  REPAIR_PROGRESS_STEPS,
  repairProgressIndex,
  repairProgressPercent,
} from "@/features/repairs/utils/repairProgress";
import type { RepairStatus } from "@/features/repairs/types/repair";
import { cn } from "@/shared/utils/cn";
import { useI18n } from "@/shared/hooks/useI18n";

type Props = {
  status: RepairStatus;
  className?: string;
};

export function RepairProgressBar({ status, className }: Props) {
  const { t } = useI18n();
  const percent = repairProgressPercent(status);
  const currentIndex = repairProgressIndex(status);
  const isCancelled = status === "cancelled";

  return (
    <div className={cn("flex flex-col gap-2", className)}>
      <div className="flex items-center justify-between gap-2 text-sm">
        <span className="font-medium text-foreground">
          {t("repairs.detail.progressLabel")}
        </span>
        <span className="text-muted">
          {isCancelled
            ? t("repairs.detail.progressCancelled")
            : t("repairs.detail.progressPercent").replace(
                "{percent}",
                String(percent),
              )}
        </span>
      </div>
      <div
        className="h-2 overflow-hidden rounded-full bg-background"
        role="progressbar"
        aria-valuenow={isCancelled ? 0 : percent}
        aria-valuemin={0}
        aria-valuemax={100}
        aria-label={t("repairs.detail.progressLabel")}
      >
        <div
          className={cn(
            "h-full rounded-full transition-[width] duration-300 ease-out",
            isCancelled ? "bg-muted" : "bg-primary",
          )}
          style={{ width: isCancelled ? "0%" : `${percent}%` }}
        />
      </div>
      {!isCancelled ? (
        <ol className="hidden gap-1 sm:grid sm:grid-cols-8">
          {REPAIR_PROGRESS_STEPS.map((step, index) => {
            const isCurrent = index === currentIndex;
            const isPast = index < currentIndex;
            return (
              <li
                key={step}
                className={cn(
                  "truncate text-center text-[10px] leading-tight sm:text-xs",
                  isCurrent && "font-semibold text-primary",
                  isPast && !isCurrent && "text-foreground",
                  !isPast && !isCurrent && "text-muted",
                )}
                title={t(`repairs.detail.progressStep.${step}`)}
              >
                {t(`repairs.detail.progressStep.${step}`)}
              </li>
            );
          })}
        </ol>
      ) : null}
    </div>
  );
}
