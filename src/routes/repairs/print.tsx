import { useParams } from "react-router";
import { RepairPrintPage } from "@/features/print/pages/RepairPrintPage";
import { useI18n } from "@/shared/hooks/useI18n";

export default function RepairsPrintRoute() {
  const { t } = useI18n();
  const params = useParams();
  const id = Number(params.id);
  if (!Number.isFinite(id) || id <= 0) {
    return <p className="p-6 text-sm text-red-700">{t("repairs.invalidId")}</p>;
  }
  return <RepairPrintPage repairId={id} />;
}
