import type { RepairIntakeState } from "@/features/repairs/hooks/useRepairIntake";
import { IntakeEstimateFields } from "@/features/repairs/components/IntakeEstimateFields";
import { RepairForm } from "@/features/repairs/components/RepairForm";

type Props = {
  intake: RepairIntakeState;
};

export function IntakeDetailsStep({ intake }: Props) {
  return (
    <div className="flex flex-col gap-6">
      <RepairForm
        form={intake.repairForm}
        lockCustomer
        lockDevice
        hideCustomerField
        hideDeviceField
        hideStatus
        hideWorkshopFields
        hideSubmit
        disabled={intake.submitting}
      />
      <IntakeEstimateFields
        form={intake.repairForm}
        disabled={intake.submitting}
        shopSettings={intake.shopSettings}
      />
    </div>
  );
}
