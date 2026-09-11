import type { useCustomerForm } from "@/features/customers/hooks/useCustomerForm";
import { Button, FormField, TextArea, TextField } from "@/ui";
import { useI18n } from "@/shared/hooks/useI18n";

type Props = {
  form: ReturnType<typeof useCustomerForm>;
  submitLabel?: string;
  disabled?: boolean;
  hideSubmit?: boolean;
};

export function CustomerForm({
  form,
  submitLabel = "",
  disabled,
  hideSubmit = false,
}: Props) {
  const { t } = useI18n();

  return (
    <form
      className="flex max-w-xl flex-col gap-4"
      onSubmit={(event) => {
        event.preventDefault();
        event.stopPropagation();
        if (!hideSubmit) {
          void form.handleSubmit();
        }
      }}
    >
      <form.Field
        name="name"
        validators={{
          onChange: ({ value }) =>
            !value.trim() ? t("customers.validation.nameRequired") : undefined,
        }}
      >
        {(field) => (
          <FormField
            label={t("customers.fields.name")}
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

      <div className="grid grid-cols-1 gap-4 sm:grid-cols-2">
        <form.Field name="phone">
          {(field) => (
            <FormField label={t("customers.fields.phone")} htmlFor={field.name}>
              <TextField
                id={field.name}
                name={field.name}
                value={field.state.value ?? ""}
                disabled={disabled}
                onBlur={field.handleBlur}
                onChange={(event) => field.handleChange(event.target.value)}
              />
            </FormField>
          )}
        </form.Field>

        <form.Field name="email">
          {(field) => (
            <FormField label={t("customers.fields.email")} htmlFor={field.name}>
              <TextField
                id={field.name}
                name={field.name}
                type="email"
                value={field.state.value ?? ""}
                disabled={disabled}
                onBlur={field.handleBlur}
                onChange={(event) => field.handleChange(event.target.value)}
              />
            </FormField>
          )}
        </form.Field>
      </div>

      <form.Field name="address">
        {(field) => (
          <FormField label={t("customers.fields.address")} htmlFor={field.name}>
            <TextField
              id={field.name}
              name={field.name}
              value={field.state.value ?? ""}
              disabled={disabled}
              onBlur={field.handleBlur}
              onChange={(event) => field.handleChange(event.target.value)}
            />
          </FormField>
        )}
      </form.Field>

      <form.Field name="notes">
        {(field) => (
          <FormField label={t("customers.fields.notes")} htmlFor={field.name}>
            <TextArea
              id={field.name}
              name={field.name}
              value={field.state.value ?? ""}
              disabled={disabled}
              onBlur={field.handleBlur}
              onChange={(event) => field.handleChange(event.target.value)}
            />
          </FormField>
        )}
      </form.Field>

      {!hideSubmit ? (
        <div>
          <form.Subscribe
            selector={(state) => [state.canSubmit, state.isSubmitting] as const}
          >
            {([canSubmit, isSubmitting]) => (
              <Button
                type="submit"
                disabled={disabled || !canSubmit || isSubmitting}
              >
                {isSubmitting ? t("common.saving") : submitLabel}
              </Button>
            )}
          </form.Subscribe>
        </div>
      ) : null}
    </form>
  );
}
