import { convertFileSrc } from "@tauri-apps/api/core";
import type { RepairPrintReport } from "@/features/print/types/printReport";
import { useI18n } from "@/shared/hooks/useI18n";

type Props = {
  report: RepairPrintReport;
};

function dash(value: string | null | undefined, empty: string): string {
  const trimmed = value?.trim();
  return trimmed && trimmed.length > 0 ? trimmed : empty;
}

function companyDisplayName(
  company: NonNullable<RepairPrintReport["company"]>,
): string {
  const trade = company.tradeName?.trim();
  return trade && trade.length > 0 ? trade : company.legalName;
}

export function RepairPrintDocument({ report }: Props) {
  const { t } = useI18n();
  const empty = t("print.emptyValue");
  const { repair, customer, device, company, companyLogoAbsolutePath } = report;
  const logoSrc = companyLogoAbsolutePath
    ? convertFileSrc(companyLogoAbsolutePath)
    : null;

  return (
    <article className="print-sheet mx-auto max-w-[210mm] bg-white text-black">
      <header className="flex items-start justify-between gap-4 border-b border-black pb-3">
        <div className="min-w-0 flex-1">
          {company ? (
            <>
              <p className="text-2xl font-semibold tracking-tight">
                {companyDisplayName(company)}
              </p>
              {company.legalName !== companyDisplayName(company) ? (
                <p className="text-sm">{company.legalName}</p>
              ) : null}
              <dl className="mt-2 grid grid-cols-[6rem_1fr] gap-x-3 gap-y-0.5 text-sm">
                {company.taxId ? (
                  <>
                    <dt>{t("print.entrance.fields.taxId")}</dt>
                    <dd>{company.taxId}</dd>
                  </>
                ) : null}
                {company.address ? (
                  <>
                    <dt>{t("print.entrance.fields.address")}</dt>
                    <dd>{company.address}</dd>
                  </>
                ) : null}
                {company.phone ? (
                  <>
                    <dt>{t("print.entrance.fields.phone")}</dt>
                    <dd>{company.phone}</dd>
                  </>
                ) : null}
                {company.email ? (
                  <>
                    <dt>{t("print.entrance.fields.email")}</dt>
                    <dd>{company.email}</dd>
                  </>
                ) : null}
                {company.website ? (
                  <>
                    <dt>{t("print.entrance.fields.website")}</dt>
                    <dd>{company.website}</dd>
                  </>
                ) : null}
              </dl>
            </>
          ) : (
            <p className="text-2xl font-semibold tracking-tight">
              {t("app.name")}
            </p>
          )}
        </div>
        {logoSrc ? (
          <img
            src={logoSrc}
            alt={t("print.entrance.logoAlt")}
            className="h-16 w-auto max-w-[40%] object-contain"
          />
        ) : null}
      </header>

      <div className="mt-4 flex flex-wrap items-baseline justify-between gap-2">
        <h1 className="text-xl font-medium">{t("print.entrance.title")}</h1>
        <p className="text-sm">
          <span className="font-medium">
            {t("print.entrance.fields.repairNumber")}:
          </span>{" "}
          {repair.repairNumber}
        </p>
      </div>
      <p className="mt-1 text-sm">
        <span className="font-medium">{t("print.entrance.fields.date")}:</span>{" "}
        {repair.receivedAt}
      </p>

      <section className="mt-4">
        <h2 className="text-sm font-semibold uppercase tracking-wide">
          {t("print.entrance.sections.customer")}
        </h2>
        <dl className="mt-2 grid grid-cols-[8rem_1fr] gap-x-3 gap-y-1 text-sm">
          <dt>{t("print.entrance.fields.name")}</dt>
          <dd>{customer.name}</dd>
          <dt>{t("print.entrance.fields.phone")}</dt>
          <dd>{dash(customer.phone, empty)}</dd>
          <dt>{t("print.entrance.fields.email")}</dt>
          <dd>{dash(customer.email, empty)}</dd>
          <dt>{t("print.entrance.fields.address")}</dt>
          <dd>{dash(customer.address, empty)}</dd>
        </dl>
      </section>

      <section className="mt-4">
        <h2 className="text-sm font-semibold uppercase tracking-wide">
          {t("print.entrance.sections.device")}
        </h2>
        <dl className="mt-2 grid grid-cols-[8rem_1fr] gap-x-3 gap-y-1 text-sm">
          <dt>{t("print.entrance.fields.deviceType")}</dt>
          <dd>{device.deviceType}</dd>
          <dt>{t("print.entrance.fields.manufacturer")}</dt>
          <dd>{dash(device.manufacturer, empty)}</dd>
          <dt>{t("print.entrance.fields.model")}</dt>
          <dd>{dash(device.model, empty)}</dd>
          <dt>{t("print.entrance.fields.serialNumber")}</dt>
          <dd>{dash(device.serialNumber, empty)}</dd>
        </dl>
      </section>

      <section className="mt-4">
        <h2 className="text-sm font-semibold uppercase tracking-wide">
          {t("print.entrance.sections.custody")}
        </h2>
        <dl className="mt-2 grid grid-cols-[8rem_1fr] gap-x-3 gap-y-1 text-sm">
          <dt>{t("print.entrance.fields.reportedProblem")}</dt>
          <dd>{dash(repair.reportedProblem, empty)}</dd>
          <dt>{t("print.entrance.fields.accessories")}</dt>
          <dd>{dash(repair.accessoriesReceived, empty)}</dd>
          <dt>{t("print.entrance.fields.condition")}</dt>
          <dd>{dash(repair.deviceCondition, empty)}</dd>
          <dt>{t("print.entrance.fields.notes")}</dt>
          <dd>{dash(repair.notes, empty)}</dd>
          <dt>{t("print.entrance.fields.expectedPickup")}</dt>
          <dd>{dash(repair.expectedPickupAt, empty)}</dd>
        </dl>
      </section>

      <section className="mt-6">
        <p className="text-sm leading-relaxed">
          {t("print.entrance.acknowledgment")}
        </p>
      </section>

      <section className="mt-10 grid grid-cols-2 gap-8">
        <div>
          <p className="text-sm font-medium">
            {t("print.entrance.signatures.client")}
          </p>
          <div className="mt-10 border-b border-black" />
          <p className="mt-1 text-xs text-neutral-600">
            {t("print.entrance.signatures.nameDate")}
          </p>
        </div>
        <div>
          <p className="text-sm font-medium">
            {t("print.entrance.signatures.shop")}
          </p>
          <div className="mt-10 border-b border-black" />
          <p className="mt-1 text-xs text-neutral-600">
            {t("print.entrance.signatures.nameDate")}
          </p>
        </div>
      </section>
    </article>
  );
}
