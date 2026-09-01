import { replace } from "react-router";
import type { Route } from "./+types/diagnosis";
import { useI18n } from "@/shared/hooks/useI18n";
import { parseEntityId } from "@/shared/utils/entityId";

/** Legacy route — opens diagnosis on the repair detail page. */
export function clientLoader({ params }: Route.ClientLoaderArgs) {
  const id = parseEntityId(params.id);
  if (!id) {
    return { invalid: true };
  }
  return replace(`/repairs/${id}?action=diagnose`);
}

export default function RepairsDiagnosisRoute({
  loaderData,
}: Route.ComponentProps) {
  const { t } = useI18n();
  if (!loaderData.invalid) {
    return null;
  }
  return <p className="p-6 text-sm text-red-700">{t("repairs.invalidId")}</p>;
}
