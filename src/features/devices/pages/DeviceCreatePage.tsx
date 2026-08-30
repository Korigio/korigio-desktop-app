import { useSearchParams } from "react-router";
import { DeviceForm } from "@/features/devices/components/DeviceForm";
import { useDeviceForm } from "@/features/devices/hooks/useDeviceForm";
import { Page, PageHeader } from "@/ui";
import { useI18n } from "@/shared/hooks/useI18n";
import { parseEntityId } from "@/shared/utils/entityId";

export function DeviceCreatePage() {
  const { t } = useI18n();
  const [params] = useSearchParams();
  const defaultCustomerId =
    parseEntityId(params.get("customerId") ?? undefined) ?? undefined;
  const form = useDeviceForm({
    mode: "create",
    defaultCustomerId,
  });

  return (
    <Page>
      <PageHeader
        title={t("devices.createTitle")}
        description={t("devices.createSubtitle")}
      />
      <DeviceForm
        form={form}
        submitLabel={t("devices.actions.create")}
        lockCustomer={Boolean(defaultCustomerId)}
      />
    </Page>
  );
}
