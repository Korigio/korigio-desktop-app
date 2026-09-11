import type { DiagnosisPrintReport } from "@/features/print/types/printReport";
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
  printFieldValue,
  printFooterLine,
} from "@/features/print/utils/printFormat";
import {
  formatMoneyCents,
  formatTaxRateBps,
} from "@/features/repairs/utils/money";
import { useI18n } from "@/shared/hooks/useI18n";

type Props = {
  report: DiagnosisPrintReport;
};

export function DiagnosisPrintDocument({ report }: Props) {
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
  const fields = (key: string) => t(`print.diagnosis.fields.${key}`);
  const title = t("print.diagnosis.title");
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
          logoAlt={t("print.diagnosis.logoAlt")}
          appNameFallback={appName}
          fieldLabels={{
            taxId: fields("taxId"),
            address: fields("address"),
            phone: fields("phone"),
            email: fields("email"),
            website: fields("website"),
          }}
        />

        <PrintDocumentHeading
          title={title}
          dateLabel={t("print.meta.date")}
          formattedDate={formatPrintDate(
            new Date().toISOString(),
            locale,
            empty,
          )}
          repairNumberLabel={fields("repairNumber")}
          repairNumber={repair.repairNumber}
        />

        <PrintRecipientBand
          customerTitle={t("print.diagnosis.sections.customer")}
          deviceTitle={t("print.diagnosis.sections.device")}
          customer={customer}
          device={device}
          empty={empty}
        />

        <PrintSection title={t("print.diagnosis.sections.findings")}>
          <PrintLongText>
            {printFieldValue(repair.diagnosisNotes, empty)}
          </PrintLongText>
        </PrintSection>

        <PrintSection title={t("print.diagnosis.sections.estimate")}>
          <PrintFieldGrid
            fields={[
              {
                label: fields("expectedPickup"),
                value: formatPrintDate(repair.expectedPickupAt, locale, empty),
              },
            ]}
          />
        </PrintSection>

        <PrintTotals
          netLabel={fields("estimateBase")}
          net={net}
          taxRateLabel={taxRateLabel}
          tax={tax}
          totalLabel={fields("gross")}
          total={total}
        />

        <PrintAcknowledgment text={t("print.diagnosis.acknowledgment")} />

        <PrintSignatures
          clientLabel={t("print.diagnosis.signatures.client")}
          shopLabel={t("print.diagnosis.signatures.shop")}
          nameDateLabel={t("print.diagnosis.signatures.nameDate")}
        />
      </PrintDocumentFrame>
    </PrintSheet>
  );
}
