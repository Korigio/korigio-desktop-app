import type { RepairPrintReport } from "@/features/print/types/printReport";
import {
  PrintAcknowledgment,
  PrintCompanyHeader,
  PrintDocumentHeading,
  PrintFieldGrid,
  PrintSection,
  PrintSheet,
  PrintSignatures,
} from "@/features/print/components/shared";
import { printFieldValue } from "@/features/print/utils/printFormat";
import { useI18n } from "@/shared/hooks/useI18n";

type Props = {
  report: RepairPrintReport;
};

export function RepairPrintDocument({ report }: Props) {
  const { t } = useI18n();
  const empty = t("print.emptyValue");
  const { repair, customer, device, company, companyLogoAbsolutePath } = report;
  const fields = (key: string) => t(`print.entrance.fields.${key}`);

  return (
    <PrintSheet>
      <PrintCompanyHeader
        company={company}
        companyLogoAbsolutePath={companyLogoAbsolutePath}
        logoAlt={t("print.entrance.logoAlt")}
        appNameFallback={t("app.name")}
        fieldLabels={{
          taxId: fields("taxId"),
          address: fields("address"),
          phone: fields("phone"),
          email: fields("email"),
          website: fields("website"),
        }}
      />

      <PrintDocumentHeading
        title={t("print.entrance.title")}
        repairNumberLabel={fields("repairNumber")}
        repairNumber={repair.repairNumber}
      />

      <p className="mt-1 text-sm">
        <span className="font-medium">{fields("date")}:</span> {repair.receivedAt}
      </p>

      <PrintSection title={t("print.entrance.sections.customer")}>
        <PrintFieldGrid
          fields={[
            { label: fields("name"), value: customer.name },
            {
              label: fields("phone"),
              value: printFieldValue(customer.phone, empty),
            },
            {
              label: fields("email"),
              value: printFieldValue(customer.email, empty),
            },
            {
              label: fields("address"),
              value: printFieldValue(customer.address, empty),
            },
          ]}
        />
      </PrintSection>

      <PrintSection title={t("print.entrance.sections.device")}>
        <PrintFieldGrid
          fields={[
            { label: fields("deviceType"), value: device.deviceType },
            {
              label: fields("manufacturer"),
              value: printFieldValue(device.manufacturer, empty),
            },
            {
              label: fields("model"),
              value: printFieldValue(device.model, empty),
            },
            {
              label: fields("serialNumber"),
              value: printFieldValue(device.serialNumber, empty),
            },
          ]}
        />
      </PrintSection>

      <PrintSection title={t("print.entrance.sections.custody")}>
        <PrintFieldGrid
          fields={[
            {
              label: fields("reportedProblem"),
              value: printFieldValue(repair.reportedProblem, empty),
            },
            {
              label: fields("accessories"),
              value: printFieldValue(repair.accessoriesReceived, empty),
            },
            {
              label: fields("condition"),
              value: printFieldValue(repair.deviceCondition, empty),
            },
            {
              label: fields("notes"),
              value: printFieldValue(repair.notes, empty),
            },
            {
              label: fields("expectedPickup"),
              value: printFieldValue(repair.expectedPickupAt, empty),
            },
          ]}
        />
      </PrintSection>

      <PrintAcknowledgment text={t("print.entrance.acknowledgment")} />

      <PrintSignatures
        clientLabel={t("print.entrance.signatures.client")}
        shopLabel={t("print.entrance.signatures.shop")}
        nameDateLabel={t("print.entrance.signatures.nameDate")}
      />
    </PrintSheet>
  );
}
