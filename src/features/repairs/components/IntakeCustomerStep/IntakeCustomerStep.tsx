import { Plus } from "lucide-react";
import type { RepairIntakeState } from "@/features/repairs/hooks/useRepairIntake";
import { CustomerForm } from "@/features/customers/components/CustomerForm";
import { CustomerSearchCombobox } from "@/features/customers/components/CustomerSearchCombobox";
import { Button, Dialog } from "@/ui";
import { useI18n } from "@/shared/hooks/useI18n";

type Props = {
  intake: RepairIntakeState;
};

export function IntakeCustomerStep({ intake }: Props) {
  const { t } = useI18n();
  const createLabel = t("repairs.intake.actions.createCustomer");

  return (
    <div className="flex flex-col gap-4">
      <div className="flex items-end gap-2">
        <div className="min-w-0 flex-1">
          <CustomerSearchCombobox
            combobox={intake.customerSearch}
            clearLabel={t("common.clearSelection")}
          />
        </div>
        <Button
          type="button"
          variant="secondary"
          className="shrink-0 px-3"
          aria-label={createLabel}
          title={createLabel}
          disabled={intake.submitting}
          onClick={intake.openCustomerCreate}
        >
          <Plus className="size-4" aria-hidden />
        </Button>
      </div>

      <Dialog
        open={intake.customerCreateOpen}
        onOpenChange={intake.onCustomerCreateOpenChange}
        title={createLabel}
        closeLabel={t("common.close")}
      >
        <CustomerForm
          form={intake.customerForm}
          submitLabel={t("customers.actions.create")}
          disabled={intake.submitting}
        />
      </Dialog>
    </div>
  );
}
