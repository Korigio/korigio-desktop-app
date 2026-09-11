import type { ReactNode } from "react";
import {
  formatMoneyCents,
  formatTaxRateBps,
  parseDiscountPercentToBps,
  parseMajorToCents,
  previewEstimateWithDiscount,
} from "@/features/repairs/utils/money";
import { useI18n } from "@/shared/hooks/useI18n";
import { FormField, StatusMessage, TextField } from "@/ui";

type EstimateFieldName = "estimateMajor" | "estimateDiscountPercent";

type EstimateFieldInstance = {
  name: string;
  state: { value: string | null | undefined };
  handleBlur: () => void;
  handleChange: (value: string) => void;
};

/**
 * Minimum TanStack Form surface for list + discount estimate fields
 * (intake create and diagnosis complete share the same shape).
 */
export type RepairEstimateFormApi = {
  Field: (props: {
    name: EstimateFieldName;
    children: (field: EstimateFieldInstance) => ReactNode;
  }) => ReactNode | Promise<ReactNode>;
  Subscribe: <TSelected>(props: {
    selector: (state: {
      values: {
        estimateMajor: string;
        estimateDiscountPercent: string;
      };
    }) => TSelected;
    children: (state: TSelected) => ReactNode;
  }) => ReactNode | Promise<ReactNode>;
};

type Props = {
  form: RepairEstimateFormApi;
  currency: string;
  /** When null, list/discount inputs still render; tax preview is omitted. */
  taxRatePercent: string | null;
  disabled?: boolean;
};

export function RepairEstimateFields({
  form,
  currency,
  taxRatePercent,
  disabled = false,
}: Props) {
  const { t } = useI18n();

  return (
    <>
      <form.Field name="estimateMajor">
        {(field) => (
          <FormField
            label={t("repairs.intake.estimate.fields.listPrice").replace(
              "{currency}",
              currency,
            )}
            htmlFor={field.name}
          >
            <TextField
              id={field.name}
              name={field.name}
              inputMode="decimal"
              value={field.state.value ?? ""}
              disabled={disabled}
              onBlur={field.handleBlur}
              onChange={(event) => field.handleChange(event.target.value)}
            />
          </FormField>
        )}
      </form.Field>

      <form.Field name="estimateDiscountPercent">
        {(field) => (
          <FormField
            label={t("repairs.intake.estimate.fields.discountPercent")}
            htmlFor={field.name}
          >
            <TextField
              id={field.name}
              name={field.name}
              inputMode="decimal"
              value={field.state.value ?? ""}
              disabled={disabled}
              onBlur={field.handleBlur}
              onChange={(event) => field.handleChange(event.target.value)}
            />
          </FormField>
        )}
      </form.Field>

      {taxRatePercent !== null ? (
        <form.Subscribe
          selector={(state) =>
            [
              state.values.estimateMajor,
              state.values.estimateDiscountPercent,
            ] as const
          }
        >
          {([estimateMajor, estimateDiscountPercent]) => {
            const listCents = estimateMajor.trim()
              ? parseMajorToCents(estimateMajor)
              : null;
            const discountRaw = estimateDiscountPercent.trim();
            const discountBps = discountRaw
              ? parseDiscountPercentToBps(discountRaw)
              : 0;

            if (!estimateMajor.trim() && !discountRaw) {
              return null;
            }

            if (
              listCents === null ||
              (discountRaw && discountBps === null) ||
              discountBps === null
            ) {
              return (
                <StatusMessage>
                  {t("repairs.diagnosisFlow.estimatePreviewInvalid")}
                </StatusMessage>
              );
            }

            const preview = previewEstimateWithDiscount(
              listCents,
              discountBps,
              taxRatePercent,
            );
            if (!preview) {
              return (
                <StatusMessage>
                  {t("repairs.diagnosisFlow.estimatePreviewInvalid")}
                </StatusMessage>
              );
            }

            return (
              <dl className="grid grid-cols-[8rem_1fr] gap-x-3 gap-y-1 text-sm">
                <dt className="text-muted">
                  {t("repairs.intake.estimate.preview.net")}
                </dt>
                <dd>{formatMoneyCents(preview.baseCents, currency)}</dd>
                <dt className="text-muted">
                  {t("repairs.diagnosisFlow.fields.taxRate")}
                </dt>
                <dd>{formatTaxRateBps(preview.taxRateBps)}%</dd>
                <dt className="text-muted">
                  {t("repairs.intake.estimate.preview.tax")}
                </dt>
                <dd>{formatMoneyCents(preview.taxCents, currency)}</dd>
                <dt className="text-muted">
                  {t("repairs.intake.estimate.preview.gross")}
                </dt>
                <dd className="font-medium">
                  {formatMoneyCents(preview.grossCents, currency)}
                </dd>
              </dl>
            );
          }}
        </form.Subscribe>
      ) : null}
    </>
  );
}
