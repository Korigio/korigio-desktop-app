import type { SummaryPrintReport } from "@/features/print/types/printReport";
import {
  PrintAcknowledgment,
  PrintCompanyHeader,
  PrintDocumentFrame,
  PrintDocumentHeading,
  PrintFieldGrid,
  PrintLongText,
  PrintRecipientBand,
  PrintSection,
  PrintSheet,
  PrintSignatures,
  PrintTotals,
} from "@/features/print/components/shared";
import {
  companyDisplayName,
  formatPrintDate,
  formatWarrantyYearsDisplay,
  printFieldValue,
  printFooterLine,
} from "@/features/print/utils/printFormat";
import {
  formatMoneyCents,
  formatTaxRateBps,
} from "@/features/repairs/utils/money";
import { useI18n } from "@/shared/hooks/useI18n";

type Props = {
  report: SummaryPrintReport;
};

export function SummaryPrintDocument({ report }: Props) {
  const { t, locale } = useI18n();
  const empty = t("print.emptyValue");
  const {
    repair,
    customer,
    device,
    company,
    companyLogoAbsolutePath,
    currency,
  } = report;
  const fields = (key: string) =>
    t(`repairs.workflow.summaryPrint.fields.${key}`);
  const sections = (key: string) =>
    t(`repairs.workflow.summaryPrint.sections.${key}`);
  const title = t("repairs.workflow.summaryPrint.title");
  const appName = t("app.name");
  const runningCompanyName = company ? companyDisplayName(company) : appName;

  const net =
    repair.estimateBaseCents != null
      ? formatMoneyCents(repair.estimateBaseCents, currency, locale)
      : empty;
  const tax =
    repair.estimateTaxCents != null
      ? formatMoneyCents(repair.estimateTaxCents, currency, locale)
      : empty;
  const total =
    repair.estimateGrossCents != null
      ? formatMoneyCents(repair.estimateGrossCents, currency, locale)
      : empty;
  const taxRateLabel =
    repair.estimateTaxRateBps != null
      ? `${fields("tax")} ${formatTaxRateBps(repair.estimateTaxRateBps)}%`
      : fields("tax");

  return (
    <PrintSheet pageLabel={t("print.page")}>
      <PrintDocumentFrame
        runningCompanyName={runningCompanyName}
        runningTitle={title}
        runningRepairNumber={repair.repairNumber}
        footerLine={printFooterLine(company) || appName}
      >
        <PrintCompanyHeader
          company={company}
          companyLogoAbsolutePath={companyLogoAbsolutePath}
          logoAlt={t("repairs.workflow.summaryPrint.logoAlt")}
          appNameFallback={appName}
          fieldLabels={{
            taxId: fields("taxId"),
            address: fields("address"),
            phone: fields("phone"),
            email: fields("email"),
            website: t("print.entrance.fields.website"),
          }}
        />

        <PrintDocumentHeading
          title={title}
          dateLabel={t("print.meta.date")}
          formattedDate={formatPrintDate(
            repair.collectedAt ?? repair.readyAt ?? repair.receivedAt,
            locale,
            empty,
          )}
          repairNumberLabel={t("repairs.workflow.summaryPrint.repairNumber")}
          repairNumber={repair.repairNumber}
        />

        <PrintRecipientBand
          customerTitle={sections("customer")}
          deviceTitle={sections("device")}
          customer={customer}
          device={device}
          empty={empty}
        />

        <PrintSection title={sections("intake")}>
          <PrintFieldGrid
            fields={[
              {
                label: fields("received"),
                value: formatPrintDate(repair.receivedAt, locale, empty),
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

        <PrintSection title={fields("reportedProblem")}>
          <PrintLongText>
            {printFieldValue(repair.reportedProblem, empty)}
          </PrintLongText>
        </PrintSection>

        <PrintSection title={sections("workPerformed")}>
          <PrintLongText>
            {printFieldValue(repair.workPerformed, empty)}
          </PrintLongText>
        </PrintSection>

        <PrintSection title={sections("pickup")}>
          <PrintFieldGrid
            fields={[
              {
                label: fields("expected"),
                value: formatPrintDate(repair.expectedPickupAt, locale, empty),
              },
              {
                label: fields("ready"),
                value: formatPrintDate(repair.readyAt, locale, empty),
              },
              {
                label: fields("collected"),
                value: formatPrintDate(repair.collectedAt, locale, empty),
              },
              {
                label: fields("warranty"),
                value: formatWarrantyYearsDisplay(
                  repair.warrantyYears,
                  empty,
                  {
                    none: t("repairs.workflow.summaryPrint.warrantyNone"),
                    one: t("repairs.workflow.summaryPrint.warrantyOne"),
                    other: t("repairs.workflow.summaryPrint.warrantyOther"),
                  },
                ),
              },
            ]}
          />
        </PrintSection>

        <PrintTotals
          netLabel={fields("subtotal")}
          net={net}
          taxRateLabel={taxRateLabel}
          tax={tax}
          totalLabel={fields("total")}
          total={total}
        />

        <PrintAcknowledgment text={t("print.summary.acknowledgment")} />

        <PrintSignatures
          clientLabel={t("print.summary.signatures.client")}
          shopLabel={t("print.summary.signatures.shop")}
          nameDateLabel={t("print.summary.signatures.nameDate")}
        />
      </PrintDocumentFrame>
    </PrintSheet>
  );
}
