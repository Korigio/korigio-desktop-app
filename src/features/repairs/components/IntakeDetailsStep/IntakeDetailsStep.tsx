import type { RepairIntakeState } from "@/features/repairs/hooks/useRepairIntake";
import { RepairForm } from "@/features/repairs/components/RepairForm";

type Props = {
  intake: RepairIntakeState;
};

export function IntakeDetailsStep({ intake }: Props) {
  return (
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
  );
}
