import type { SummaryPrintReport } from "@/features/print/types/printReport";
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
import {
  formatMoneyCents,
  formatTaxRateBps,
} from "@/features/repairs/utils/money";
import { useI18n } from "@/shared/hooks/useI18n";

type Props = {
  report: SummaryPrintReport;
};

export function SummaryPrintDocument({ report }: Props) {
  const { t } = useI18n();
  const empty = t("print.emptyValue");
  const { repair, customer, device, company, companyLogoAbsolutePath, currency } =
    report;
  const fields = (key: string) =>
    t(`repairs.workflow.summaryPrint.fields.${key}`);
  const sections = (key: string) =>
    t(`repairs.workflow.summaryPrint.sections.${key}`);

  return (
    <PrintSheet>
      <PrintCompanyHeader
        company={company}
        companyLogoAbsolutePath={companyLogoAbsolutePath}
        logoAlt={t("repairs.workflow.summaryPrint.logoAlt")}
        appNameFallback={t("app.name")}
        fieldLabels={{
          taxId: fields("taxId"),
          address: fields("address"),
          phone: fields("phone"),
          email: fields("email"),
        }}
      />

      <PrintDocumentHeading
        title={t("repairs.workflow.summaryPrint.title")}
        repairNumberLabel={t("repairs.workflow.summaryPrint.repairNumber")}
        repairNumber={repair.repairNumber}
      />

      <PrintSection title={sections("customer")}>
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

      <PrintSection title={sections("device")}>
        <PrintFieldGrid
          fields={[
            { label: fields("type"), value: device.deviceType },
            {
              label: fields("manufacturer"),
              value: printFieldValue(device.manufacturer, empty),
            },
            {
              label: fields("model"),
              value: printFieldValue(device.model, empty),
            },
            {
              label: fields("serial"),
              value: printFieldValue(device.serialNumber, empty),
            },
          ]}
        />
      </PrintSection>

      <PrintSection title={sections("intake")}>
        <PrintFieldGrid
          fields={[
            { label: fields("received"), value: repair.receivedAt.slice(0, 10) },
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
          ]}
        />
      </PrintSection>

      <PrintSection title={sections("workPerformed")}>
        <p className="mt-2 whitespace-pre-wrap text-sm">
          {printFieldValue(repair.workPerformed, empty)}
        </p>
      </PrintSection>

      <PrintSection title={sections("estimate")}>
        <PrintFieldGrid
          fields={[
            {
              label: fields("subtotal"),
              value:
                repair.estimateBaseCents != null
                  ? formatMoneyCents(repair.estimateBaseCents, currency)
                  : empty,
            },
            {
              label: fields("taxRate"),
              value:
                repair.estimateTaxRateBps != null
                  ? `${formatTaxRateBps(repair.estimateTaxRateBps)}%`
                  : empty,
            },
            {
              label: fields("tax"),
              value:
                repair.estimateTaxCents != null
                  ? formatMoneyCents(repair.estimateTaxCents, currency)
                  : empty,
            },
            {
              label: fields("total"),
              value:
                repair.estimateGrossCents != null
                  ? formatMoneyCents(repair.estimateGrossCents, currency)
                  : empty,
            },
          ]}
        />
      </PrintSection>

      <PrintSection title={sections("pickup")}>
        <PrintFieldGrid
          fields={[
            {
              label: fields("expected"),
              value: printFieldValue(repair.expectedPickupAt, empty),
            },
            {
              label: fields("ready"),
              value: repair.readyAt ? repair.readyAt.slice(0, 10) : empty,
            },
            {
              label: fields("collected"),
              value: repair.collectedAt ? repair.collectedAt.slice(0, 10) : empty,
            },
          ]}
        />
      </PrintSection>

      <PrintAcknowledgment text={t("print.summary.acknowledgment")} />

      <PrintSignatures
        clientLabel={t("print.summary.signatures.client")}
        shopLabel={t("print.summary.signatures.shop")}
        nameDateLabel={t("print.summary.signatures.nameDate")}
      />
    </PrintSheet>
  );
}
