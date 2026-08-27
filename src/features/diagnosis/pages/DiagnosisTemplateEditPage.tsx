import { DiagnosisTemplateForm } from "@/features/diagnosis/components/DiagnosisTemplateForm";
import { useDiagnosisTemplateDetail } from "@/features/diagnosis/hooks/useDiagnosisTemplateDetail";
import { useDiagnosisTemplateForm } from "@/features/diagnosis/hooks/useDiagnosisTemplateForm";
import type { DiagnosisTemplate } from "@/features/diagnosis/types/diagnosis";
import { Page, PageHeader, StatusMessage } from "@/ui";
import { useI18n } from "@/shared/hooks/useI18n";

type Props = { templateId: number };

export function DiagnosisTemplateEditPage({ templateId }: Props) {
  const { t } = useI18n();
  const { template, loading, error } = useDiagnosisTemplateDetail(templateId);

  if (loading) {
    return (
      <Page>
        <StatusMessage>{t("common.loading")}</StatusMessage>
      </Page>
    );
  }

  if (error || !template) {
    return (
      <Page>
        <StatusMessage tone="danger">
          {error ?? t("diagnosis.notFound")}
        </StatusMessage>
      </Page>
    );
  }

  return <DiagnosisTemplateEditForm template={template} />;
}

function DiagnosisTemplateEditForm({
  template,
}: {
  template: DiagnosisTemplate;
}) {
  const { t } = useI18n();
  const form = useDiagnosisTemplateForm({ mode: "edit", template });

  return (
    <Page>
      <PageHeader
        title={t("diagnosis.editTitle")}
        description={template.name}
      />
      <DiagnosisTemplateForm
        form={form}
        submitLabel={t("diagnosis.actions.save")}
      />
    </Page>
  );
}
