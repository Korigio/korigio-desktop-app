import { useEffect, useState } from "react";
import type { Repair } from "@/features/repairs/types/repair";
import {
  formatMoneyCents,
  formatTaxRateBps,
} from "@/features/repairs/utils/money";
import {
  isDiagnosisComplete,
  isDiagnosisPending,
  isIntakePending,
  isRepairCompleted,
  isRepairDiagnosisLocked,
  repairDiagnosisActionKey,
} from "@/features/repairs/utils/repairDiagnosis";
import { settingsApi } from "@/features/settings/api/settingsApi";
import { Card, DefinitionList, Button, LinkButton, StatusMessage } from "@/ui";
import { useI18n } from "@/shared/hooks/useI18n";

const EMPTY = "—";

type Props = {
  repair: Repair;
  onOpenDiagnosis?: () => void;
};

export function RepairDiagnosisSummary({ repair, onOpenDiagnosis }: Props) {
  const { t } = useI18n();
  const [currency, setCurrency] = useState("EUR");
  const actionKey = repairDiagnosisActionKey(repair);

  useEffect(() => {
    let cancelled = false;
    void settingsApi
      .getShopSettings()
      .then((settings) => {
        if (!cancelled) setCurrency(settings.currency);
      })
      .catch(() => {
        /* keep default currency for display */
      });
    return () => {
      cancelled = true;
    };
  }, []);

  const hasContent =
    Boolean(repair.diagnosisNotes?.trim()) || repair.estimateBaseCents != null;

  const estimateValue =
    repair.estimateBaseCents != null
      ? [
          formatMoneyCents(repair.estimateBaseCents, currency),
          repair.estimateTaxCents != null && repair.estimateTaxRateBps != null
            ? t("repairs.diagnosisFlow.summary.estimateTaxLine")
                .replace(
                  "{tax}",
                  formatMoneyCents(repair.estimateTaxCents, currency),
                )
                .replace("{rate}", formatTaxRateBps(repair.estimateTaxRateBps))
            : null,
          repair.estimateGrossCents != null
            ? t("repairs.diagnosisFlow.summary.estimateGrossLine").replace(
                "{gross}",
                formatMoneyCents(repair.estimateGrossCents, currency),
              )
            : null,
        ]
          .filter(Boolean)
          .join(" · ")
      : EMPTY;

  const statusHint = isRepairCompleted(repair)
    ? t("repairs.detail.diagnosisLockedHint")
    : isIntakePending(repair)
      ? t("repairs.detail.intakePendingHint")
      : isDiagnosisPending(repair)
        ? t("repairs.detail.diagnosisPendingHint")
        : isDiagnosisComplete(repair) && !isRepairDiagnosisLocked(repair)
          ? t("repairs.detail.diagnosisCompleteHint")
          : null;

  const canPrintDiagnosis =
    hasContent && !isIntakePending(repair) && !isDiagnosisPending(repair);

  return (
    <Card
      title={t("repairs.detail.sections.diagnosis")}
      actions={
        <div className="flex flex-wrap gap-2">
          {canPrintDiagnosis ? (
            <LinkButton
              to={`/repairs/${repair.id}/print/diagnosis`}
              variant="secondary"
            >
              {t("repairs.diagnosisFlow.actions.print")}
            </LinkButton>
          ) : null}
          {actionKey && onOpenDiagnosis ? (
            <Button type="button" onClick={onOpenDiagnosis}>
              {t(`repairs.actions.${actionKey}`)}
            </Button>
          ) : null}
        </div>
      }
    >
      {statusHint ? (
        <p className="mb-4 text-sm text-muted">{statusHint}</p>
      ) : null}

      {!hasContent ? (
        <StatusMessage>
          {t("repairs.diagnosisFlow.summary.empty")}
        </StatusMessage>
      ) : (
        <DefinitionList
          items={[
            {
              label: t("repairs.fields.diagnosisNotes"),
              value: repair.diagnosisNotes?.trim() || EMPTY,
            },
            {
              label: t("repairs.diagnosisFlow.fields.estimate"),
              value: estimateValue,
            },
          ]}
        />
      )}
    </Card>
  );
}
