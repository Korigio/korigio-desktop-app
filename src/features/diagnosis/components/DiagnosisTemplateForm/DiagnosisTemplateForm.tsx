import type { useDiagnosisTemplateForm } from "@/features/diagnosis/hooks/useDiagnosisTemplateForm";
import { newTemplateItemId } from "@/features/diagnosis/types/diagnosis";
import {
  Button,
  Card,
  FormField,
  SelectField,
  StatusMessage,
  TextField,
} from "@/ui";
import { useI18n } from "@/shared/hooks/useI18n";

type Props = {
  form: ReturnType<typeof useDiagnosisTemplateForm>["form"];
  submitError?: string | null;
  submitLabel: string;
  disabled?: boolean;
};

export function DiagnosisTemplateForm({
  form,
  submitError = null,
  submitLabel,
  disabled = false,
}: Props) {
  const { t } = useI18n();

  return (
    <form
      className="flex flex-col gap-4"
      onSubmit={(event) => {
        event.preventDefault();
        event.stopPropagation();
        void form.handleSubmit();
      }}
    >
      {submitError ? (
        <StatusMessage tone="danger">{submitError}</StatusMessage>
      ) : null}

      <Card title={t("diagnosis.detail.sections.name")}>
        <form.Field
          name="name"
          validators={{
            onChange: ({ value }) =>
              !value.trim()
                ? t("diagnosis.validation.nameRequired")
                : undefined,
          }}
        >
          {(field) => (
            <FormField
              label={t("diagnosis.fields.name")}
              htmlFor={field.name}
              error={
                typeof field.state.meta.errors[0] === "string"
                  ? field.state.meta.errors[0]
                  : undefined
              }
            >
              <TextField
                id={field.name}
                name={field.name}
                value={field.state.value}
                disabled={disabled}
                onBlur={field.handleBlur}
                onChange={(event) => field.handleChange(event.target.value)}
              />
            </FormField>
          )}
        </form.Field>
      </Card>

      <form.Field
        name="items"
        mode="array"
        validators={{
          onChange: ({ value }) =>
            value.length === 0
              ? t("diagnosis.validation.itemsRequired")
              : undefined,
        }}
      >
        {(itemsField) => (
          <Card
            title={t("diagnosis.detail.sections.items")}
            actions={
              <Button
                type="button"
                variant="secondary"
                className="px-2 py-1 text-xs"
                disabled={disabled}
                onClick={() =>
                  itemsField.pushValue({
                    id: newTemplateItemId(),
                    label: "",
                    kind: "checkbox",
                  })
                }
              >
                {t("diagnosis.actions.addItem")}
              </Button>
            }
          >
            <div className="flex flex-col gap-3">
              {typeof itemsField.state.meta.errors[0] === "string" ? (
                <p className="text-sm text-red-700">
                  {itemsField.state.meta.errors[0]}
                </p>
              ) : null}

              {itemsField.state.value.map((item, index) => (
                <div
                  key={item.id}
                  className="flex flex-col gap-3 rounded-md border border-border p-3"
                >
                  <form.Field
                    name={`items[${index}].label`}
                    validators={{
                      onChange: ({ value }) =>
                        !value.trim()
                          ? t("diagnosis.validation.labelRequired")
                          : undefined,
                    }}
                  >
                    {(field) => (
                      <FormField
                        label={t("diagnosis.fields.itemLabel")}
                        htmlFor={field.name}
                        error={
                          typeof field.state.meta.errors[0] === "string"
                            ? field.state.meta.errors[0]
                            : undefined
                        }
                      >
                        <TextField
                          id={field.name}
                          name={field.name}
                          value={field.state.value}
                          disabled={disabled}
                          onBlur={field.handleBlur}
                          onChange={(event) =>
                            field.handleChange(event.target.value)
                          }
                        />
                      </FormField>
                    )}
                  </form.Field>

                  <form.Field name={`items[${index}].kind`}>
                    {(field) => (
                      <FormField
                        label={t("diagnosis.fields.itemKind")}
                        htmlFor={field.name}
                      >
                        <SelectField
                          id={field.name}
                          name={field.name}
                          value={field.state.value}
                          disabled={disabled}
                          onBlur={field.handleBlur}
                          onChange={(event) =>
                            field.handleChange(
                              event.target.value as "checkbox" | "text",
                            )
                          }
                        >
                          <option value="checkbox">
                            {t("diagnosis.kinds.checkbox")}
                          </option>
                          <option value="text">
                            {t("diagnosis.kinds.text")}
                          </option>
                        </SelectField>
                      </FormField>
                    )}
                  </form.Field>

                  <div>
                    <Button
                      type="button"
                      variant="secondary"
                      className="px-2 py-1 text-xs"
                      disabled={disabled || itemsField.state.value.length <= 1}
                      onClick={() => itemsField.removeValue(index)}
                    >
                      {t("diagnosis.actions.removeItem")}
                    </Button>
                  </div>
                </div>
              ))}
            </div>
          </Card>
        )}
      </form.Field>

      <div>
        <form.Subscribe selector={(state) => state.isSubmitting}>
          {(isSubmitting) => (
            <Button type="submit" disabled={disabled || isSubmitting}>
              {isSubmitting ? t("common.saving") : submitLabel}
            </Button>
          )}
        </form.Subscribe>
      </div>
    </form>
  );
}
