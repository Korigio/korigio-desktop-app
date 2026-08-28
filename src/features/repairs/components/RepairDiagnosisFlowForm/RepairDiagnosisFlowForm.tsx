import type { RepairDiagnosisFlowState } from "@/features/repairs/hooks/useRepairDiagnosisFlow";
import { isDiagnosisComplete } from "@/features/repairs/utils/repairDiagnosis";
import {
  formatMoneyCents,
  formatTaxRateBps,
  parseMajorToCents,
  previewEstimate,
} from "@/features/repairs/utils/money";
import {
  Button,
  CheckboxField,
  FormField,
  LinkButton,
  SelectField,
  StatusMessage,
  TextArea,
  TextField,
} from "@/ui";
import { useI18n } from "@/shared/hooks/useI18n";

type Props = {
  flow: RepairDiagnosisFlowState;
  variant?: "page" | "modal";
};

export function RepairDiagnosisFlowForm({ flow, variant = "page" }: Props) {
  const { t } = useI18n();
  const {
    form,
    shopSettings,
    templates,
    selectedTemplateId,
    setSelectedTemplateId,
    checklistItems,
    setChecklistItemValue,
    loadingTemplate,
    saving,
    estimateLocked,
    canPrint,
    loadTemplateChecklist,
    applyChecklistToNotes,
    saveDraft,
    finalize,
    repair,
  } = flow;

  if (!repair || !shopSettings) {
    return null;
  }

  const currency = shopSettings.currency;
  const isArchived = Boolean(repair.archivedAt);
  const isCancelled = repair.status === "cancelled";
  const readOnly = isArchived || isCancelled;
  const adjusting = isDiagnosisComplete(repair);

  return (
    <div className={variant === "modal" ? "flex flex-col gap-6" : "flex max-w-xl flex-col gap-6"}>
      {readOnly ? (
        <StatusMessage>
          {isArchived
            ? t("repairs.diagnosisFlow.archivedHint")
            : t("repairs.diagnosisFlow.cancelledHint")}
        </StatusMessage>
      ) : null}

      <form.Field name="diagnosisNotes">
        {(field) => (
          <FormField
            label={t("repairs.diagnosisFlow.fields.findings")}
            htmlFor={field.name}
          >
            <TextArea
              id={field.name}
              name={field.name}
              rows={8}
              value={field.state.value ?? ""}
              disabled={readOnly || saving}
              onBlur={field.handleBlur}
              onChange={(event) => field.handleChange(event.target.value)}
            />
          </FormField>
        )}
      </form.Field>

      <section className="flex flex-col gap-3">
        <h2 className="text-sm font-medium">
          {t("repairs.diagnosisFlow.templateHeading")}
        </h2>
        {templates.length === 0 ? (
          <StatusMessage>{t("repairs.diagnosisFlow.noTemplates")}</StatusMessage>
        ) : (
          <>
            <FormField
              label={t("repairs.diagnosisFlow.fields.template")}
              htmlFor="diagnosis-flow-template"
            >
              <SelectField
                id="diagnosis-flow-template"
                value={selectedTemplateId}
                disabled={readOnly || saving || loadingTemplate}
                onChange={(event) => setSelectedTemplateId(event.target.value)}
              >
                {templates.map((template) => (
                  <option key={template.id} value={String(template.id)}>
                    {template.name}
                  </option>
                ))}
              </SelectField>
            </FormField>
            <div>
              <Button
                type="button"
                variant="secondary"
                disabled={
                  readOnly ||
                  saving ||
                  loadingTemplate ||
                  !selectedTemplateId
                }
                onClick={() => void loadTemplateChecklist()}
              >
                {loadingTemplate
                  ? t("common.loading")
                  : t("repairs.diagnosisFlow.actions.loadTemplate")}
              </Button>
            </div>
          </>
        )}

        {checklistItems.length > 0 ? (
          <div className="flex flex-col gap-3">
            {checklistItems.map((item, index) =>
              item.kind === "checkbox" ? (
                <CheckboxField
                  key={item.id}
                  id={`diagnosis-flow-item-${item.id}`}
                  label={item.label}
                  checked={Boolean(item.value)}
                  disabled={readOnly || saving}
                  onChange={(event) =>
                    setChecklistItemValue(index, event.target.checked)
                  }
                />
              ) : (
                <FormField
                  key={item.id}
                  label={item.label}
                  htmlFor={`diagnosis-flow-item-${item.id}`}
                >
                  <TextField
                    id={`diagnosis-flow-item-${item.id}`}
                    value={typeof item.value === "string" ? item.value : ""}
                    disabled={readOnly || saving}
                    onChange={(event) =>
                      setChecklistItemValue(index, event.target.value)
                    }
                  />
                </FormField>
              ),
            )}
            <div>
              <Button
                type="button"
                variant="secondary"
                disabled={readOnly || saving}
                onClick={applyChecklistToNotes}
              >
                {t("repairs.diagnosisFlow.actions.applyToNotes")}
              </Button>
            </div>
          </div>
        ) : null}
      </section>

      <section className="flex flex-col gap-3">
        <h2 className="text-sm font-medium">
          {t("repairs.diagnosisFlow.estimateHeading")}
        </h2>
        <form.Field name="estimateMajor">
          {(field) => (
            <>
              <FormField
                label={t("repairs.diagnosisFlow.fields.estimateBase").replace(
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
                  disabled={readOnly || saving}
                  onBlur={field.handleBlur}
                  onChange={(event) => field.handleChange(event.target.value)}
                />
              </FormField>
              <p className="text-sm text-muted">
                {estimateLocked
                  ? t("repairs.diagnosisFlow.estimateLockedHint")
                  : t("repairs.diagnosisFlow.estimateHint")}
              </p>
            </>
          )}
        </form.Field>

        <form.Subscribe selector={(state) => state.values.estimateMajor}>
          {(estimateMajor) => {
            const cents = parseMajorToCents(estimateMajor);
            const preview =
              cents !== null
                ? previewEstimate(cents, shopSettings.taxRatePercent)
                : null;
            if (!preview) {
              return estimateMajor.trim() ? (
                <StatusMessage>
                  {t("repairs.diagnosisFlow.estimatePreviewInvalid")}
                </StatusMessage>
              ) : null;
            }
            return (
              <dl className="grid grid-cols-[8rem_1fr] gap-x-3 gap-y-1 text-sm">
                <dt className="text-muted">
                  {t("repairs.diagnosisFlow.fields.taxRate")}
                </dt>
                <dd>
                  {formatTaxRateBps(preview.taxRateBps)}%
                </dd>
                <dt className="text-muted">
                  {t("repairs.diagnosisFlow.fields.tax")}
                </dt>
                <dd>{formatMoneyCents(preview.taxCents, currency)}</dd>
                <dt className="text-muted">
                  {t("repairs.diagnosisFlow.fields.gross")}
                </dt>
                <dd className="font-medium">
                  {formatMoneyCents(preview.grossCents, currency)}
                </dd>
              </dl>
            );
          }}
        </form.Subscribe>
      </section>

      <form.Field name="expectedPickupAt">
        {(field) => (
          <FormField
            label={t("repairs.fields.expectedPickupAt")}
            htmlFor={field.name}
          >
            <TextField
              id={field.name}
              name={field.name}
              type="date"
              value={field.state.value ?? ""}
              disabled={readOnly || saving}
              onBlur={field.handleBlur}
              onChange={(event) => field.handleChange(event.target.value)}
            />
          </FormField>
        )}
      </form.Field>

      <div className="flex flex-wrap gap-2">
        <Button
          type="button"
          variant={adjusting ? "primary" : "secondary"}
          disabled={readOnly || saving}
          onClick={saveDraft}
        >
          {saving
            ? t("common.saving")
            : adjusting
              ? t("repairs.diagnosisFlow.actions.saveChanges")
              : t("repairs.diagnosisFlow.actions.saveDraft")}
        </Button>
        {!adjusting ? (
          <Button
            type="button"
            disabled={readOnly || saving}
            onClick={finalize}
          >
            {saving
              ? t("common.saving")
              : t("repairs.diagnosisFlow.actions.finalize")}
          </Button>
        ) : null}
        <LinkButton
          to={`/repairs/${repair.id}/print/diagnosis`}
          variant="secondary"
          className={canPrint ? undefined : "pointer-events-none opacity-50"}
          aria-disabled={!canPrint}
          onClick={(event) => {
            if (!canPrint) event.preventDefault();
          }}
        >
          {t("repairs.diagnosisFlow.actions.print")}
        </LinkButton>
        {variant === "page" ? (
          <LinkButton to={`/repairs/${repair.id}`} variant="secondary">
            {t("repairs.diagnosisFlow.actions.backToRepair")}
          </LinkButton>
        ) : null}
      </div>
    </div>
  );
}
