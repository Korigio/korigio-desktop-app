import { DiagnosisTemplateForm } from "@/features/diagnosis/components/DiagnosisTemplateForm";
import { useDiagnosisTemplateForm } from "@/features/diagnosis/hooks/useDiagnosisTemplateForm";
import { Page, PageHeader } from "@/ui";
import { useI18n } from "@/shared/hooks/useI18n";

export function DiagnosisTemplateCreatePage() {
  const { t } = useI18n();
  const { form, submitError } = useDiagnosisTemplateForm({ mode: "create" });

  return (
    <Page className="max-w-5xl">
      <PageHeader
        title={t("diagnosis.createTitle")}
        description={t("diagnosis.detail.subtitle")}
      />
      <DiagnosisTemplateForm
        form={form}
        submitError={submitError}
        submitLabel={t("diagnosis.actions.create")}
      />
    </Page>
  );
}
