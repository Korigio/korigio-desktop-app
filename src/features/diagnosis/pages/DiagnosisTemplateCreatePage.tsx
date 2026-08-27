import { DiagnosisTemplateForm } from "@/features/diagnosis/components/DiagnosisTemplateForm";
import { useDiagnosisTemplateForm } from "@/features/diagnosis/hooks/useDiagnosisTemplateForm";
import { Page, PageHeader } from "@/ui";
import { useI18n } from "@/shared/hooks/useI18n";

export function DiagnosisTemplateCreatePage() {
  const { t } = useI18n();
  const form = useDiagnosisTemplateForm({ mode: "create" });

  return (
    <Page>
      <PageHeader
        title={t("diagnosis.createTitle")}
        description={t("diagnosis.createSubtitle")}
      />
      <DiagnosisTemplateForm
        form={form}
        submitLabel={t("diagnosis.actions.create")}
      />
    </Page>
  );
}
