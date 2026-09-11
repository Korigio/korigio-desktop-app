import { Plus } from "lucide-react";
import type { RepairIntakeState } from "@/features/repairs/hooks/useRepairIntake";
import { DeviceForm } from "@/features/devices/components/DeviceForm";
import { DeviceSearchCombobox } from "@/features/devices/components/DeviceSearchCombobox";
import { Button, Dialog } from "@/ui";
import { useI18n } from "@/shared/hooks/useI18n";
import { customerLabel } from "@/features/customers/hooks/useCustomerSearchCombobox";

type Props = {
  intake: RepairIntakeState;
};

export function IntakeDeviceStep({ intake }: Props) {
  const { t } = useI18n();
  const createLabel = t("repairs.intake.actions.createDevice");

  if (intake.isNewCustomer) {
    return (
      <DeviceForm
        form={intake.deviceForm}
        submitLabel={t("devices.actions.create")}
        lockCustomer
        hideCustomerField
        hideSubmit
        lockedCustomerLabel={
          intake.customer ? customerLabel(intake.customer) : undefined
        }
        disabled={intake.submitting}
      />
    );
  }

  return (
    <div className="flex flex-col gap-4">
      <div className="flex items-end gap-2">
        <div className="min-w-0 flex-1">
          <DeviceSearchCombobox
            combobox={intake.deviceSearch}
            clearLabel={t("common.clearSelection")}
          />
        </div>
        <Button
          type="button"
          variant="secondary"
          className="size-10 shrink-0 px-0"
          aria-label={createLabel}
          title={createLabel}
          disabled={intake.submitting || !intake.customer}
          onClick={intake.openDeviceCreate}
        >
          <Plus className="size-4" aria-hidden />
        </Button>
      </div>

      <Dialog
        open={intake.deviceCreateOpen}
        onOpenChange={intake.onDeviceCreateOpenChange}
        title={createLabel}
        closeLabel={t("common.close")}
      >
        <DeviceForm
          form={intake.deviceForm}
          submitLabel={t("devices.actions.create")}
          lockCustomer
          hideCustomerField
          lockedCustomerLabel={
            intake.customer ? customerLabel(intake.customer) : undefined
          }
          disabled={intake.submitting}
        />
      </Dialog>
    </div>
  );
}
