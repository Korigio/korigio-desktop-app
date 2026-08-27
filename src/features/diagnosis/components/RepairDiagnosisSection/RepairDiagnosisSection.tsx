import { useRepairDiagnosis } from "@/features/diagnosis/hooks/useRepairDiagnosis";
import {
  Button,
  CheckboxField,
  FormField,
  PageHeader,
  SelectField,
  StatusMessage,
  TextField,
} from "@/ui";
import { useI18n } from "@/shared/hooks/useI18n";

type Props = {
  repairId: number;
};

export function RepairDiagnosisSection({ repairId }: Props) {
  const { t } = useI18n();
  const diagnosisState = useRepairDiagnosis(repairId);

  return (
    <section className="flex flex-col gap-4">
      <PageHeader title={t("diagnosis.repairSectionTitle")} />

      {diagnosisState.error ? (
        <StatusMessage tone="danger">{diagnosisState.error}</StatusMessage>
      ) : null}

      {diagnosisState.success ? (
        <StatusMessage tone="success">{diagnosisState.success}</StatusMessage>
      ) : null}

      {diagnosisState.loading ? (
        <StatusMessage>{t("common.loading")}</StatusMessage>
      ) : diagnosisState.diagnosis === null ? (
        <div className="flex max-w-xl flex-col gap-3">
          <p className="text-sm text-muted">{t("diagnosis.repairEmpty")}</p>
          {diagnosisState.templates.length === 0 ? (
            <StatusMessage>{t("diagnosis.noTemplates")}</StatusMessage>
          ) : (
            <>
              <FormField
                label={t("diagnosis.fields.template")}
                htmlFor="repair-diagnosis-template"
              >
                <SelectField
                  id="repair-diagnosis-template"
                  value={diagnosisState.selectedTemplateId}
                  onChange={(event) =>
                    diagnosisState.setSelectedTemplateId(event.target.value)
                  }
                >
                  {diagnosisState.templates.map((template) => (
                    <option key={template.id} value={String(template.id)}>
                      {template.name}
                    </option>
                  ))}
                </SelectField>
              </FormField>
              <div>
                <Button
                  type="button"
                  disabled={
                    diagnosisState.applying ||
                    !diagnosisState.selectedTemplateId
                  }
                  onClick={() => void diagnosisState.applyTemplate()}
                >
                  {diagnosisState.applying
                    ? t("common.saving")
                    : t("diagnosis.actions.applyTemplate")}
                </Button>
              </div>
            </>
          )}
        </div>
      ) : (
        <div className="flex max-w-xl flex-col gap-4">
          {diagnosisState.draftItems.map((item, index) =>
            item.kind === "checkbox" ? (
              <CheckboxField
                key={item.id}
                id={`diagnosis-item-${item.id}`}
                label={item.label}
                checked={Boolean(item.value)}
                onChange={(event) =>
                  diagnosisState.setItemValue(index, event.target.checked)
                }
              />
            ) : (
              <FormField
                key={item.id}
                label={item.label}
                htmlFor={`diagnosis-item-${item.id}`}
              >
                <TextField
                  id={`diagnosis-item-${item.id}`}
                  value={typeof item.value === "string" ? item.value : ""}
                  onChange={(event) =>
                    diagnosisState.setItemValue(index, event.target.value)
                  }
                />
              </FormField>
            ),
          )}
          <div>
            <Button
              type="button"
              disabled={diagnosisState.saving}
              onClick={() => void diagnosisState.save()}
            >
              {diagnosisState.saving
                ? t("common.saving")
                : t("diagnosis.actions.save")}
            </Button>
          </div>
        </div>
      )}
    </section>
  );
}
