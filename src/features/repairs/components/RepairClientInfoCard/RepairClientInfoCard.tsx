import type { Company } from "@/features/companies/types/company";
import { companyLabel } from "@/features/companies/types/company";
import type { Customer } from "@/features/customers/types/customer";
import { customerLabel } from "@/features/customers/hooks/useCustomerSearchCombobox";
import type { Device } from "@/features/devices/types/device";
import { deviceLabel } from "@/features/devices/types/device";
import type { Repair } from "@/features/repairs/types/repair";
import { Card, DefinitionList, LinkButton, StatusMessage } from "@/ui";
import { useI18n } from "@/shared/hooks/useI18n";

const EMPTY = "—";

type Props = {
  repair: Repair;
  customer: Customer | null;
  device: Device | null;
  company: Company | null;
  loading?: boolean;
};

export function RepairClientInfoCard({
  repair,
  customer,
  device,
  company,
  loading = false,
}: Props) {
  const { t } = useI18n();

  return (
    <Card title={t("repairs.detail.sections.client")}>
      {loading ? (
        <StatusMessage>{t("common.loading")}</StatusMessage>
      ) : (
        <DefinitionList
          items={[
            ...(company
              ? [
                  {
                    label: t("repairs.fields.company"),
                    value: companyLabel(company),
                  },
                ]
              : []),
            {
              label: t("repairs.fields.customer"),
              value: customer ? (
                <div className="flex flex-col gap-0.5">
                  <LinkButton
                    to={`/customers/${repair.customerId}`}
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
                  to={`/customers/${repair.customerId}`}
                  variant="secondary"
                  className="px-2 py-1 text-xs"
                >
                  #{repair.customerId}
                </LinkButton>
              ),
            },
            {
              label: t("repairs.fields.device"),
              value: device ? (
                <LinkButton
                  to={`/devices/${repair.deviceId}`}
                  variant="secondary"
                  className="h-auto w-fit px-0 py-0 text-sm font-medium"
                >
                  {deviceLabel(device)}
                </LinkButton>
              ) : (
                <LinkButton
                  to={`/devices/${repair.deviceId}`}
                  variant="secondary"
                  className="px-2 py-1 text-xs"
                >
                  #{repair.deviceId}
                </LinkButton>
              ),
            },
          ]}
        />
      )}
    </Card>
  );
}
