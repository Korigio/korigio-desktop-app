import type { RepairPrintReport } from "@/features/print/types/printReport";
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
} from "@/features/print/components/shared";
import {
  companyDisplayName,
  formatPrintDate,
  printFieldValue,
  printFooterLine,
} from "@/features/print/utils/printFormat";
import { useI18n } from "@/shared/hooks/useI18n";

type Props = {
  report: RepairPrintReport;
};

export function RepairPrintDocument({ report }: Props) {
  const { t, locale } = useI18n();
  const empty = t("print.emptyValue");
  const { repair, customer, device, company, companyLogoAbsolutePath } = report;
  const fields = (key: string) => t(`print.entrance.fields.${key}`);
  const title = t("print.entrance.title");
  const appName = t("app.name");
  const runningCompanyName = company ? companyDisplayName(company) : appName;

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
          logoAlt={t("print.entrance.logoAlt")}
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
          formattedDate={formatPrintDate(repair.receivedAt, locale, empty)}
          repairNumberLabel={fields("repairNumber")}
          repairNumber={repair.repairNumber}
        />

        <PrintRecipientBand
          customerTitle={t("print.entrance.sections.customer")}
          deviceTitle={t("print.entrance.sections.device")}
          customer={customer}
          device={device}
          empty={empty}
        />

        <PrintSection title={t("print.entrance.sections.custody")}>
          <PrintFieldGrid
            fields={[
              {
                label: fields("accessories"),
                value: printFieldValue(repair.accessoriesReceived, empty),
              },
              {
                label: fields("condition"),
                value: printFieldValue(repair.deviceCondition, empty),
              },
              {
                label: fields("expectedPickup"),
                value: formatPrintDate(repair.expectedPickupAt, locale, empty),
              },
            ]}
          />
        </PrintSection>

        <PrintSection title={fields("reportedProblem")}>
          <PrintLongText>
            {printFieldValue(repair.reportedProblem, empty)}
          </PrintLongText>
        </PrintSection>

        <PrintSection title={fields("notes")}>
          <PrintLongText>{printFieldValue(repair.notes, empty)}</PrintLongText>
        </PrintSection>

        <PrintAcknowledgment text={t("print.entrance.acknowledgment")} />

        <PrintSignatures
          clientLabel={t("print.entrance.signatures.client")}
          shopLabel={t("print.entrance.signatures.shop")}
          nameDateLabel={t("print.entrance.signatures.nameDate")}
        />
      </PrintDocumentFrame>
    </PrintSheet>
  );
}
