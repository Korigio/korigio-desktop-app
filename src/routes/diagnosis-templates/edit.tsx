import { useParams } from "react-router";
import { DiagnosisTemplateEditPage } from "@/features/diagnosis/pages/DiagnosisTemplateEditPage";
import { useI18n } from "@/shared/hooks/useI18n";
import { parseEntityId } from "@/shared/utils/entityId";

export default function DiagnosisTemplatesEditRoute() {
  const { t } = useI18n();
  const params = useParams();
  const id = parseEntityId(params.id);
  if (!id) {
    return (
      <p className="p-6 text-sm text-red-700">{t("diagnosis.invalidId")}</p>
    );
  }
  return <DiagnosisTemplateEditPage templateId={id} />;
}
