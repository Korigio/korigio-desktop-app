import { useParams } from "react-router";
import { RepairDetailPage } from "@/features/repairs/pages/RepairDetailPage";
import { useI18n } from "@/shared/hooks/useI18n";
import { parseEntityId } from "@/shared/utils/entityId";

export default function RepairsDetailRoute() {
  const { t } = useI18n();
  const params = useParams();
  const id = parseEntityId(params.id);
  if (!id) {
    return <p className="p-6 text-sm text-red-700">{t("repairs.invalidId")}</p>;
  }
  return <RepairDetailPage repairId={id} />;
}
