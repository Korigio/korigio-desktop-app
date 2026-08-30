import { useParams } from "react-router";
import { SummaryPrintPage } from "@/features/print/pages/SummaryPrintPage";
import { useI18n } from "@/shared/hooks/useI18n";
import { parseEntityId } from "@/shared/utils/entityId";

export default function RepairsPrintSummaryRoute() {
  const { t } = useI18n();
  const params = useParams();
  const id = parseEntityId(params.id);
  if (!id) {
    return <p className="p-6 text-sm text-red-700">{t("repairs.invalidId")}</p>;
  }
  return <SummaryPrintPage repairId={id} />;
}
