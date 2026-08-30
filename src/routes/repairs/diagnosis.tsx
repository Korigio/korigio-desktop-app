import { Navigate, useParams } from "react-router";
import { useI18n } from "@/shared/hooks/useI18n";
import { parseEntityId } from "@/shared/utils/entityId";

/** Legacy route — opens diagnosis on the repair detail page. */
export default function RepairsDiagnosisRoute() {
  const { t } = useI18n();
  const params = useParams();
  const id = parseEntityId(params.id);
  if (!id) {
    return <p className="p-6 text-sm text-red-700">{t("repairs.invalidId")}</p>;
  }
  return <Navigate to={`/repairs/${id}?action=diagnose`} replace />;
}
