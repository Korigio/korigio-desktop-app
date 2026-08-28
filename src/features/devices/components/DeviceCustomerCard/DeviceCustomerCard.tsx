import { customerLabel } from "@/features/customers/hooks/useCustomerSearchCombobox";
import type { Customer } from "@/features/customers/types/customer";
import type { Device } from "@/features/devices/types/device";
import { Card, DefinitionList, LinkButton, StatusMessage } from "@/ui";
import { useI18n } from "@/shared/hooks/useI18n";

const EMPTY = "—";

type Props = {
  device: Device;
  customer: Customer | null;
  loading?: boolean;
};

export function DeviceCustomerCard({
  device,
  customer,
  loading = false,
}: Props) {
  const { t } = useI18n();

  return (
    <Card title={t("devices.detail.sections.customer")}>
      {loading ? (
        <StatusMessage>{t("common.loading")}</StatusMessage>
      ) : (
        <DefinitionList
          items={[
            {
              label: t("devices.fields.customer"),
              value: customer ? (
                <div className="flex flex-col gap-0.5">
                  <LinkButton
                    to={`/customers/${device.customerId}`}
                    variant="secondary"
                    className="h-auto w-fit px-0 py-0 text-sm font-medium"
                  >
                    {customerLabel(customer)}
                  </LinkButton>
                  <span className="text-sm text-muted">
                    {[
                      customer.phone
                        ? `${t("customers.fields.phone")}: ${customer.phone}`
                        : null,
                      customer.email
                        ? `${t("customers.fields.email")}: ${customer.email}`
                        : null,
                    ]
                      .filter(Boolean)
                      .join(" · ") || EMPTY}
                  </span>
                </div>
              ) : (
                <LinkButton
                  to={`/customers/${device.customerId}`}
                  variant="secondary"
                  className="px-2 py-1 text-xs"
                >
                  #{device.customerId}
                </LinkButton>
              ),
            },
          ]}
        />
      )}
    </Card>
  );
}
