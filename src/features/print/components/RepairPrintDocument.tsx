import type { RepairPrintReport } from "@/features/print/types/printReport";
import { useI18n } from "@/shared/hooks/useI18n";

type Props = {
  report: RepairPrintReport;
};

function dash(value: string | null | undefined, empty: string): string {
  const trimmed = value?.trim();
  return trimmed && trimmed.length > 0 ? trimmed : empty;
}

export function RepairPrintDocument({ report }: Props) {
  const { t } = useI18n();
  const empty = t("print.emptyValue");
  const { repair, customer, device, diagnosis } = report;

  return (
    <article className="print-sheet mx-auto max-w-[210mm] bg-white text-black">
      <header className="border-b border-black pb-3">
        <p className="text-2xl font-semibold tracking-tight">{t("app.name")}</p>
        <h1 className="mt-1 text-xl font-medium">
          {t("print.title")} · {repair.repairNumber}
        </h1>
        <p className="mt-1 text-sm">
          {t("print.fields.status")}: {t(`repairs.status.${repair.status}`)}
        </p>
      </header>

      <section className="mt-4">
        <h2 className="text-sm font-semibold uppercase tracking-wide">
          {t("print.sections.customer")}
        </h2>
        <dl className="mt-2 grid grid-cols-[8rem_1fr] gap-x-3 gap-y-1 text-sm">
          <dt>{t("print.fields.name")}</dt>
          <dd>{customer.name}</dd>
          <dt>{t("print.fields.phone")}</dt>
          <dd>{dash(customer.phone, empty)}</dd>
          <dt>{t("print.fields.email")}</dt>
          <dd>{dash(customer.email, empty)}</dd>
          <dt>{t("print.fields.address")}</dt>
          <dd>{dash(customer.address, empty)}</dd>
        </dl>
      </section>

      <section className="mt-4">
        <h2 className="text-sm font-semibold uppercase tracking-wide">
          {t("print.sections.device")}
        </h2>
        <dl className="mt-2 grid grid-cols-[8rem_1fr] gap-x-3 gap-y-1 text-sm">
          <dt>{t("print.fields.deviceType")}</dt>
          <dd>{device.deviceType}</dd>
          <dt>{t("print.fields.manufacturer")}</dt>
          <dd>{dash(device.manufacturer, empty)}</dd>
          <dt>{t("print.fields.model")}</dt>
          <dd>{dash(device.model, empty)}</dd>
          <dt>{t("print.fields.serialNumber")}</dt>
          <dd>{dash(device.serialNumber, empty)}</dd>
        </dl>
      </section>

      <section className="mt-4">
        <h2 className="text-sm font-semibold uppercase tracking-wide">
          {t("print.sections.repair")}
        </h2>
        <dl className="mt-2 grid grid-cols-[8rem_1fr] gap-x-3 gap-y-1 text-sm">
          <dt>{t("print.fields.receivedAt")}</dt>
          <dd>{repair.receivedAt}</dd>
          <dt>{t("print.fields.reportedProblem")}</dt>
          <dd>{dash(repair.reportedProblem, empty)}</dd>
          <dt>{t("print.fields.accessoriesReceived")}</dt>
          <dd>{dash(repair.accessoriesReceived, empty)}</dd>
          <dt>{t("print.fields.deviceCondition")}</dt>
          <dd>{dash(repair.deviceCondition, empty)}</dd>
          <dt>{t("repairs.fields.expectedPickupAt")}</dt>
          <dd>{dash(repair.expectedPickupAt, empty)}</dd>
          <dt>{t("print.fields.diagnosisNotes")}</dt>
          <dd>{dash(repair.diagnosisNotes, empty)}</dd>
          <dt>{t("print.fields.workPerformed")}</dt>
          <dd>{dash(repair.workPerformed, empty)}</dd>
          <dt>{t("print.fields.notes")}</dt>
          <dd>{dash(repair.notes, empty)}</dd>
          <dt>{t("print.fields.readyAt")}</dt>
          <dd>{dash(repair.readyAt, empty)}</dd>
          <dt>{t("print.fields.collectedAt")}</dt>
          <dd>{dash(repair.collectedAt, empty)}</dd>
        </dl>
      </section>

      <section className="mt-4">
        <h2 className="text-sm font-semibold uppercase tracking-wide">
          {t("print.sections.diagnosis")}
        </h2>
        {diagnosis && diagnosis.items.length > 0 ? (
          <ul className="mt-2 list-none space-y-1 text-sm">
            {diagnosis.items.map((item) => (
              <li key={item.id} className="flex gap-2 border-b border-neutral-300 py-1">
                <span className="min-w-0 flex-1">{item.label}</span>
                <span className="shrink-0 font-medium">
                  {typeof item.value === "boolean"
                    ? item.value
                      ? t("print.yes")
                      : t("print.no")
                    : dash(item.value, empty)}
                </span>
              </li>
            ))}
          </ul>
        ) : (
          <p className="mt-2 text-sm">{t("print.noDiagnosis")}</p>
        )}
      </section>
    </article>
  );
}
