import { useEffect, useState } from "react";
import { customersApi } from "@/features/customers/api/customersApi";
import type { Customer } from "@/features/customers/types/customer";
import { devicesApi } from "@/features/devices/api/devicesApi";
import { deviceLabel, type Device } from "@/features/devices/types/device";
import type { useRepairForm } from "@/features/repairs/hooks/useRepairForm";
import { REPAIR_STATUSES } from "@/features/repairs/types/repair";
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
  form: ReturnType<typeof useRepairForm>;
  submitLabel?: string;
  disabled?: boolean;
  lockCustomer?: boolean;
  lockDevice?: boolean;
  hideSubmit?: boolean;
  hideStatus?: boolean;
  hideWorkshopFields?: boolean;
  hideCustomerField?: boolean;
  hideDeviceField?: boolean;
  lockedCustomerLabel?: string;
  lockedDeviceLabel?: string;
};

export function RepairForm({
  form,
  submitLabel = "",
  disabled = false,
  lockCustomer = false,
  lockDevice = false,
  hideSubmit = false,
  hideStatus = false,
  hideWorkshopFields = false,
  hideCustomerField = false,
  hideDeviceField = false,
  lockedCustomerLabel,
  lockedDeviceLabel,
}: Props) {
  const { t } = useI18n();
  const [customers, setCustomers] = useState<Customer[]>([]);
  const [customersError, setCustomersError] = useState<string | null>(null);
  const [devices, setDevices] = useState<Device[]>([]);
  const [devicesError, setDevicesError] = useState<string | null>(null);
  const [selectedCustomerId, setSelectedCustomerId] = useState(
    () => form.state.values.customerId ?? "",
  );

  useEffect(() => {
    if (lockCustomer || hideCustomerField) {
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
  }, [lockCustomer, hideCustomerField]);

  useEffect(() => {
    if (lockDevice || hideDeviceField) {
      return;
    }
    const customerId = Number(selectedCustomerId);
    if (!Number.isFinite(customerId) || customerId <= 0) {
      setDevices([]);
      return;
    }
    let cancelled = false;
    void devicesApi
      .list({ customerId, includeArchived: false, page: 1, pageSize: 100 })
      .then((result) => {
        if (!cancelled) {
          setDevices(result.items);
        }
      })
      .catch((err: unknown) => {
        if (!cancelled) {
          setDevicesError(
            err instanceof Error ? err.message : "Failed to load devices",
          );
        }
      });
    return () => {
      cancelled = true;
    };
  }, [selectedCustomerId, lockDevice, hideDeviceField]);

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
      {!hideCustomerField ? (
        <form.Field
          name="customerId"
          validators={{
            onChange: ({ value }) =>
              !value.trim()
                ? t("repairs.validation.customerRequired")
                : undefined,
          }}
        >
          {(field) => (
            <FormField
              label={t("repairs.fields.customer")}
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
                  value={
                    lockedCustomerLabel?.trim() ||
                    (field.state.value ? `#${field.state.value}` : "")
                  }
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
                  onChange={(event) => {
                    const nextCustomerId = event.target.value;
                    field.handleChange(nextCustomerId);
                    form.setFieldValue("deviceId", "");
                    setSelectedCustomerId(nextCustomerId);
                  }}
                >
                  <option value="">
                    {t("repairs.fields.customerPlaceholder")}
                  </option>
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
      ) : null}

      {customersError ? (
        <StatusMessage tone="danger">{customersError}</StatusMessage>
      ) : null}

      {!hideDeviceField ? (
        <form.Field
          name="deviceId"
          validators={{
            onChange: ({ value }) =>
              !value.trim()
                ? t("repairs.validation.deviceRequired")
                : undefined,
          }}
        >
          {(field) => (
            <FormField
              label={t("repairs.fields.device")}
              htmlFor={field.name}
              error={
                typeof field.state.meta.errors[0] === "string"
                  ? field.state.meta.errors[0]
                  : undefined
              }
            >
              {lockDevice ? (
                <TextField
                  id={field.name}
                  name={field.name}
                  value={
                    lockedDeviceLabel?.trim() ||
                    (field.state.value ? `#${field.state.value}` : "")
                  }
                  disabled
                  readOnly
                />
              ) : (
                <SelectField
                  id={field.name}
                  name={field.name}
                  value={field.state.value}
                  disabled={disabled || !selectedCustomerId}
                  onBlur={field.handleBlur}
                  onChange={(event) => field.handleChange(event.target.value)}
                >
                  <option value="">
                    {t("repairs.fields.devicePlaceholder")}
                  </option>
                  {devices.map((device) => (
                    <option key={device.id} value={String(device.id)}>
                      {deviceLabel(device)}
                    </option>
                  ))}
                </SelectField>
              )}
            </FormField>
          )}
        </form.Field>
      ) : null}

      {devicesError ? (
        <StatusMessage tone="danger">{devicesError}</StatusMessage>
      ) : null}

      {!hideStatus ? (
        <form.Field name="status">
          {(field) => (
            <FormField label={t("repairs.fields.status")} htmlFor={field.name}>
              <SelectField
                id={field.name}
                name={field.name}
                value={field.state.value}
                disabled={disabled}
                onBlur={field.handleBlur}
                onChange={(event) => field.handleChange(event.target.value)}
              >
                {REPAIR_STATUSES.map((code) => (
                  <option key={code} value={code}>
                    {t(`repairs.status.${code}`)}
                  </option>
                ))}
              </SelectField>
            </FormField>
          )}
        </form.Field>
      ) : null}

      <form.Field name="reportedProblem">
        {(field) => (
          <FormField
            label={t("repairs.fields.reportedProblem")}
            htmlFor={field.name}
          >
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

      <form.Field name="accessoriesReceived">
        {(field) => (
          <FormField
            label={t("repairs.fields.accessoriesReceived")}
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

      <form.Field name="deviceCondition">
        {(field) => (
          <FormField
            label={t("repairs.fields.deviceCondition")}
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
              disabled={disabled}
              onBlur={field.handleBlur}
              onChange={(event) => field.handleChange(event.target.value)}
            />
          </FormField>
        )}
      </form.Field>

      {!hideWorkshopFields ? (
        <>
          <form.Field name="diagnosisNotes">
            {(field) => (
              <FormField
                label={t("repairs.fields.diagnosisNotes")}
                htmlFor={field.name}
              >
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

          <form.Field name="workPerformed">
            {(field) => (
              <FormField
                label={t("repairs.fields.workPerformed")}
                htmlFor={field.name}
              >
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
        </>
      ) : null}

      <form.Field name="notes">
        {(field) => (
          <FormField label={t("repairs.fields.notes")} htmlFor={field.name}>
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
