import type { useRepairIntake } from "@/features/repairs/hooks/useRepairIntake";
import {
  Button,
  FormField,
  SearchCombobox,
  StatusMessage,
  TextArea,
  TextField,
} from "@/ui";
import { useI18n } from "@/shared/hooks/useI18n";

type Props = {
  intake: ReturnType<typeof useRepairIntake>;
};

export function RepairIntakeForm({ intake }: Props) {
  const { t } = useI18n();

  return (
    <div className="flex max-w-xl flex-col gap-4">
      {intake.successMessage ? (
        <StatusMessage tone="success">
          {t("repairs.intake.success").replace(
            "{number}",
            intake.successMessage,
          )}
        </StatusMessage>
      ) : null}

      {intake.error ? (
        <StatusMessage tone="danger">{intake.error}</StatusMessage>
      ) : null}

      <SearchCombobox
        inputRef={intake.customerInputRef}
        label={t("repairs.fields.customer")}
        query={intake.customerQuery}
        onQueryChange={intake.setCustomerQuery}
        items={intake.customers}
        renderItem={(item) => intake.customerLabel(item)}
        getItemKey={(item) => item.id}
        onSelect={intake.selectCustomer}
        selectedLabel={intake.customer ? intake.customerLabel(intake.customer) : null}
        onClearSelection={intake.clearCustomer}
        loading={intake.customersLoading}
        emptyMessage={t("repairs.intake.noCustomers")}
        footerAction={{
          label: t("repairs.intake.quickCreateCustomer"),
          onClick: intake.openQuickCreateCustomer,
        }}
      />

      {intake.showQuickCreateCustomer ? (
        <section className="rounded-md border border-border bg-background p-4">
          <p className="mb-3 text-sm font-medium">
            {t("repairs.intake.quickCreateCustomerTitle")}
          </p>
          <div className="flex flex-col gap-3">
            <FormField
              label={t("customers.fields.name")}
              htmlFor="quick-customer-name"
            >
              <TextField
                id="quick-customer-name"
                value={intake.quickCustomerName}
                onChange={(event) =>
                  intake.setQuickCustomerName(event.target.value)
                }
              />
            </FormField>
            <FormField
              label={t("customers.fields.phone")}
              htmlFor="quick-customer-phone"
            >
              <TextField
                id="quick-customer-phone"
                value={intake.quickCustomerPhone}
                onChange={(event) =>
                  intake.setQuickCustomerPhone(event.target.value)
                }
              />
            </FormField>
            <div className="flex gap-2">
              <Button
                type="button"
                disabled={intake.submitting}
                onClick={() => void intake.createQuickCustomer()}
              >
                {t("customers.actions.create")}
              </Button>
              <Button
                type="button"
                variant="secondary"
                onClick={() => intake.setShowQuickCreateCustomer(false)}
              >
                {t("repairs.intake.cancelQuickCreate")}
              </Button>
            </div>
          </div>
        </section>
      ) : null}

      <SearchCombobox
        label={t("repairs.fields.device")}
        query={intake.deviceQuery}
        onQueryChange={intake.setDeviceQuery}
        items={intake.devices}
        renderItem={(item) => intake.deviceLabel(item)}
        getItemKey={(item) => item.id}
        onSelect={intake.selectDevice}
        selectedLabel={intake.device ? intake.deviceLabel(intake.device) : null}
        onClearSelection={intake.clearDevice}
        loading={intake.devicesLoading}
        disabled={!intake.customer}
        emptyMessage={
          intake.customer
            ? t("repairs.intake.noDevices")
            : t("repairs.intake.selectCustomerFirst")
        }
        footerAction={
          intake.customer
            ? {
                label: t("repairs.intake.quickCreateDevice"),
                onClick: intake.openQuickCreateDevice,
              }
            : undefined
        }
      />

      {intake.showQuickCreateDevice ? (
        <section className="rounded-md border border-border bg-background p-4">
          <p className="mb-3 text-sm font-medium">
            {t("repairs.intake.quickCreateDeviceTitle")}
          </p>
          <div className="flex flex-col gap-3">
            <FormField
              label={t("devices.fields.deviceType")}
              htmlFor="quick-device-type"
            >
              <TextField
                id="quick-device-type"
                value={intake.quickDeviceType}
                onChange={(event) =>
                  intake.setQuickDeviceType(event.target.value)
                }
              />
            </FormField>
            <FormField
              label={t("devices.fields.serialNumber")}
              htmlFor="quick-device-serial"
            >
              <TextField
                id="quick-device-serial"
                value={intake.quickDeviceSerial}
                onChange={(event) =>
                  intake.setQuickDeviceSerial(event.target.value)
                }
              />
            </FormField>
            <div className="flex gap-2">
              <Button
                type="button"
                disabled={intake.submitting}
                onClick={() => void intake.createQuickDevice()}
              >
                {t("devices.actions.create")}
              </Button>
              <Button
                type="button"
                variant="secondary"
                onClick={() => intake.setShowQuickCreateDevice(false)}
              >
                {t("repairs.intake.cancelQuickCreate")}
              </Button>
            </div>
          </div>
        </section>
      ) : null}

      <FormField
        label={t("repairs.fields.reportedProblem")}
        htmlFor="intake-reported-problem"
      >
        <TextArea
          ref={intake.problemInputRef}
          id="intake-reported-problem"
          rows={3}
          value={intake.reportedProblem}
          onChange={(event) => intake.setReportedProblem(event.target.value)}
        />
      </FormField>

      <FormField
        label={t("repairs.fields.accessoriesReceived")}
        htmlFor="intake-accessories"
      >
        <TextField
          id="intake-accessories"
          value={intake.accessoriesReceived}
          onChange={(event) =>
            intake.setAccessoriesReceived(event.target.value)
          }
        />
      </FormField>

      <FormField
        label={t("repairs.fields.deviceCondition")}
        htmlFor="intake-condition"
      >
        <TextField
          id="intake-condition"
          value={intake.deviceCondition}
          onChange={(event) => intake.setDeviceCondition(event.target.value)}
        />
      </FormField>

      <div className="flex flex-wrap gap-2">
        <Button
          type="button"
          disabled={intake.submitting}
          onClick={() => void intake.submit()}
        >
          {t("repairs.intake.saveAndNext")}
        </Button>
      </div>

      <p className="text-xs text-muted">{t("repairs.intake.shortcutHint")}</p>
    </div>
  );
}
