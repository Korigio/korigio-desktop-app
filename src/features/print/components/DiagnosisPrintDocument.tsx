import type { DiagnosisPrintReport } from "@/features/print/types/printReport";
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
  report: DiagnosisPrintReport;
};

export function DiagnosisPrintDocument({ report }: Props) {
  const { t } = useI18n();
  const empty = t("print.emptyValue");
  const { repair, customer, device, company, companyLogoAbsolutePath, currency } =
    report;
  const fields = (key: string) => t(`print.diagnosis.fields.${key}`);

  return (
    <PrintSheet>
      <PrintCompanyHeader
        company={company}
        companyLogoAbsolutePath={companyLogoAbsolutePath}
        logoAlt={t("print.diagnosis.logoAlt")}
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
        title={t("print.diagnosis.title")}
        repairNumberLabel={fields("repairNumber")}
        repairNumber={repair.repairNumber}
      />

      <PrintSection title={t("print.diagnosis.sections.customer")}>
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

      <PrintSection title={t("print.diagnosis.sections.device")}>
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

      <PrintSection title={t("print.diagnosis.sections.findings")}>
        <p className="mt-2 whitespace-pre-wrap text-sm">
          {printFieldValue(repair.diagnosisNotes, empty)}
        </p>
      </PrintSection>

      <PrintSection title={t("print.diagnosis.sections.estimate")}>
        <PrintFieldGrid
          fields={[
            {
              label: fields("estimateBase"),
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
              label: fields("gross"),
              value:
                repair.estimateGrossCents != null
                  ? formatMoneyCents(repair.estimateGrossCents, currency)
                  : empty,
            },
            { label: fields("currency"), value: currency },
            {
              label: fields("expectedPickup"),
              value: printFieldValue(repair.expectedPickupAt, empty),
            },
          ]}
        />
      </PrintSection>

      <PrintAcknowledgment text={t("print.diagnosis.acknowledgment")} />

      <PrintSignatures
        clientLabel={t("print.diagnosis.signatures.client")}
        shopLabel={t("print.diagnosis.signatures.shop")}
        nameDateLabel={t("print.diagnosis.signatures.nameDate")}
      />
    </PrintSheet>
  );
}
