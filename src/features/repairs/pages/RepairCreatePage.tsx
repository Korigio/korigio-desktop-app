import { useSearchParams } from "react-router";
import { RepairForm } from "@/features/repairs/components/RepairForm";
import { useRepairForm } from "@/features/repairs/hooks/useRepairForm";
import { Page, PageHeader } from "@/ui";
import { useI18n } from "@/shared/hooks/useI18n";

function parseId(raw: string | null): number | undefined {
  const id = Number(raw);
  return Number.isFinite(id) && id > 0 ? id : undefined;
}

export function RepairCreatePage() {
  const { t } = useI18n();
  const [params] = useSearchParams();
  const defaultCustomerId = parseId(params.get("customerId"));
  const defaultDeviceId = parseId(params.get("deviceId"));
  const form = useRepairForm({
    mode: "create",
    defaultCustomerId,
    defaultDeviceId,
  });

  return (
    <Page>
      <PageHeader
        title={t("repairs.createTitle")}
        description={t("repairs.createSubtitle")}
      />
      <RepairForm
        form={form}
        submitLabel={t("repairs.actions.create")}
        lockCustomer={Boolean(defaultCustomerId)}
        lockDevice={Boolean(defaultDeviceId)}
      />
    </Page>
  );
}
