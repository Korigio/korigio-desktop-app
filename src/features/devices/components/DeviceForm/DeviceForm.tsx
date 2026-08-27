import { useEffect, useState } from "react";
import { customersApi } from "@/features/customers/api/customersApi";
import type { Customer } from "@/features/customers/types/customer";
import type { useDeviceForm } from "@/features/devices/hooks/useDeviceForm";
import {
  Button,
  FormField,
  SelectField,
  StatusMessage,
  TextArea,
  TextField,
} from "@/ui";
import { useI18n } from "@/shared/hooks/useI18n";

type Props = {
  form: ReturnType<typeof useDeviceForm>;
  submitLabel: string;
  disabled?: boolean;
  lockCustomer?: boolean;
};

export function DeviceForm({
  form,
  submitLabel,
  disabled = false,
  lockCustomer = false,
}: Props) {
  const { t } = useI18n();
  const [customers, setCustomers] = useState<Customer[]>([]);
  const [customersError, setCustomersError] = useState<string | null>(null);

  useEffect(() => {
    if (lockCustomer) {
      return;
    }
    let cancelled = false;
    void customersApi
      .list({ includeArchived: false, page: 1, pageSize: 100 })
      .then((result) => {
        if (!cancelled) {
          setCustomers(result.items);
        }
      })
      .catch((err: unknown) => {
        if (!cancelled) {
          setCustomersError(
            err instanceof Error ? err.message : "Failed to load customers",
          );
        }
      });
    return () => {
      cancelled = true;
    };
  }, [lockCustomer]);

  return (
    <form
      className="flex max-w-xl flex-col gap-4"
      onSubmit={(event) => {
        event.preventDefault();
        event.stopPropagation();
        void form.handleSubmit();
      }}
    >
      <form.Field
        name="customerId"
        validators={{
          onChange: ({ value }) =>
            !value.trim() ? t("devices.validation.customerRequired") : undefined,
        }}
      >
        {(field) => (
          <FormField
            label={t("devices.fields.customer")}
            htmlFor={field.name}
            error={
              typeof field.state.meta.errors[0] === "string"
                ? field.state.meta.errors[0]
                : undefined
            }
          >
            {lockCustomer ? (
              <TextField
                id={field.name}
                name={field.name}
                value={`#${field.state.value}`}
                disabled
                readOnly
              />
            ) : (
              <SelectField
                id={field.name}
                name={field.name}
                value={field.state.value}
                disabled={disabled}
                onBlur={field.handleBlur}
                onChange={(event) => field.handleChange(event.target.value)}
              >
                <option value="">{t("devices.fields.customerPlaceholder")}</option>
                {customers.map((customer) => (
                  <option key={customer.id} value={String(customer.id)}>
                    {customer.name}
                  </option>
                ))}
              </SelectField>
            )}
          </FormField>
        )}
      </form.Field>

      {customersError ? (
        <StatusMessage tone="danger">{customersError}</StatusMessage>
      ) : null}

      <form.Field name="deviceType">
        {(field) => (
          <FormField label={t("devices.fields.deviceType")} htmlFor={field.name}>
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

      <form.Field name="manufacturer">
        {(field) => (
          <FormField
            label={t("devices.fields.manufacturer")}
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

      <form.Field name="model">
        {(field) => (
          <FormField label={t("devices.fields.model")} htmlFor={field.name}>
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

      <form.Field name="serialNumber">
        {(field) => (
          <FormField
            label={t("devices.fields.serialNumber")}
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

      <form.Field name="accessories">
        {(field) => (
          <FormField
            label={t("devices.fields.accessories")}
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

      <form.Field name="notes">
        {(field) => (
          <FormField label={t("devices.fields.notes")} htmlFor={field.name}>
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

      <p className="text-sm text-muted">{t("devices.validation.identityHint")}</p>

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
    </form>
  );
}
