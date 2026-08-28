import type { Device } from "@/features/devices/types/device";
import { Card } from "@/ui";
import { useI18n } from "@/shared/hooks/useI18n";
import { cn } from "@/shared/utils/cn";

type Props = {
  device: Device;
};

export function DeviceDetailStatusPanel({ device }: Props) {
  const { t } = useI18n();
  const isArchived = Boolean(device.archivedAt);

  return (
    <Card className="border-primary/20 bg-surface">
      <div className="flex flex-col gap-4 sm:flex-row sm:items-start sm:justify-between">
        <div className="flex min-w-0 flex-col gap-1">
          <p className="text-xs font-medium uppercase tracking-wide text-muted">
            {t("devices.fields.status")}
          </p>
          <p
            className={cn(
              "text-2xl font-semibold sm:text-3xl",
              isArchived ? "text-muted" : "text-foreground",
            )}
          >
            {isArchived
              ? t("devices.status.archived")
              : t("devices.status.active")}
          </p>
        </div>
        <div className="flex flex-col gap-1 text-sm text-muted sm:text-right">
          <p>
            {t("devices.fields.createdAt")}: {device.createdAt.slice(0, 10)}
          </p>
          <p>
            {t("devices.fields.updatedAt")}: {device.updatedAt.slice(0, 10)}
          </p>
        </div>
      </div>
    </Card>
  );
}
