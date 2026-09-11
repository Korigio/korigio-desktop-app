import type { ReactNode } from "react";
import { Button, FormField, TextField } from "@/ui";
import { useI18n } from "@/shared/hooks/useI18n";

type CompanyFieldName =
  | "legalName"
  | "tradeName"
  | "taxId"
  | "address"
  | "phone"
  | "email"
  | "website";

type CompanyFieldInstance = {
  name: string;
  state: {
    value: string | null | undefined;
    meta: { errors: unknown[] };
  };
  handleBlur: () => void;
  handleChange: (value: string) => void;
};

/**
 * Minimum TanStack Form surface CompanyForm needs so first-run (extra
 * tax/currency fields) can pass the same instance as create/edit.
 */
export type CompanyFormApi = {
  handleSubmit: () => unknown;
  Field: (props: {
    name: CompanyFieldName;
    validators?: {
      onChange?: (opts: { value: string }) => string | undefined;
    };
    children: (field: CompanyFieldInstance) => ReactNode;
  }) => ReactNode | Promise<ReactNode>;
  Subscribe: <TSelected>(props: {
    selector: (state: {
      canSubmit: boolean;
      isSubmitting: boolean;
    }) => TSelected;
    children: (state: TSelected) => ReactNode;
  }) => ReactNode | Promise<ReactNode>;
};

type Props = {
  form: CompanyFormApi;
  submitLabel?: string;
  disabled?: boolean;
  hideSubmit?: boolean;
  children?: ReactNode;
};

export function CompanyForm({
  form,
  submitLabel = "",
  disabled,
  hideSubmit = false,
  children,
}: Props) {
  const { t } = useI18n();

  return (
    <form
      className="grid max-w-xl grid-cols-1 gap-4 sm:grid-cols-2"
      onSubmit={(event) => {
        event.preventDefault();
        event.stopPropagation();
        void form.handleSubmit();
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
            className="sm:col-span-2"
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
              value={field.state.value ?? ""}
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
            className="sm:col-span-2"
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
            className="sm:col-span-2"
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

      {children ? (
        <div className="flex flex-col gap-4 sm:col-span-2">{children}</div>
      ) : null}

      {!hideSubmit ? (
        <div className="sm:col-span-2">
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
