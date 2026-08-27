import type { useCompanyForm } from "@/features/companies/hooks/useCompanyForm";
import { Button, FormField, TextField } from "@/ui";
import { useI18n } from "@/shared/hooks/useI18n";

type Props = {
  form: ReturnType<typeof useCompanyForm>;
  submitLabel?: string;
  disabled?: boolean;
  hideSubmit?: boolean;
};

export function CompanyForm({
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
        name="legalName"
        validators={{
          onChange: ({ value }) =>
            !value.trim()
              ? t("companies.validation.legalNameRequired")
              : undefined,
        }}
      >
        {(field) => (
          <FormField
            label={t("companies.fields.legalName")}
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

      <form.Field name="tradeName">
        {(field) => (
          <FormField
            label={t("companies.fields.tradeName")}
            htmlFor={field.name}
          >
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

      <form.Field name="taxId">
        {(field) => (
          <FormField label={t("companies.fields.taxId")} htmlFor={field.name}>
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

      <form.Field name="address">
        {(field) => (
          <FormField
            label={t("companies.fields.address")}
            htmlFor={field.name}
          >
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

      <form.Field name="phone">
        {(field) => (
          <FormField label={t("companies.fields.phone")} htmlFor={field.name}>
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
          <FormField label={t("companies.fields.email")} htmlFor={field.name}>
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

      <form.Field name="website">
        {(field) => (
          <FormField
            label={t("companies.fields.website")}
            htmlFor={field.name}
          >
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
