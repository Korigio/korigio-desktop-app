import type { DiagnosisTemplate } from "@/features/diagnosis/types/diagnosis";
import { Card } from "@/ui";
import { useI18n } from "@/shared/hooks/useI18n";

type Props = {
  template: DiagnosisTemplate;
};

export function DiagnosisTemplateStatusPanel({ template }: Props) {
  const { t } = useI18n();
  const itemCount = template.body.items.length;

  return (
    <Card className="border-primary/20 bg-surface">
      <div className="flex flex-col gap-4 sm:flex-row sm:items-start sm:justify-between">
        <div className="flex min-w-0 flex-col gap-1">
          <p className="text-xs font-medium uppercase tracking-wide text-muted">
            {t("diagnosis.detail.sections.items")}
          </p>
          <p className="text-2xl font-semibold text-foreground sm:text-3xl">
            {t("diagnosis.detail.itemCount").replace(
              "{count}",
              String(itemCount),
            )}
          </p>
        </div>
        <div className="flex flex-col gap-1 text-sm text-muted sm:text-right">
          <p>
            {t("diagnosis.fields.updatedAt")}: {template.updatedAt.slice(0, 10)}
          </p>
        </div>
      </div>
    </Card>
  );
}
