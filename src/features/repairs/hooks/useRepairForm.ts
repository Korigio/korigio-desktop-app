import { useForm } from "@tanstack/react-form";
import { useNavigate } from "react-router";
import { repairsApi } from "@/features/repairs/api/repairsApi";
import type {
  Repair,
  RepairInput,
  RepairStatus,
} from "@/features/repairs/types/repair";

type FormValues = {
  customerId: string;
  deviceId: string;
  status: string;
  reportedProblem: string;
  accessoriesReceived: string;
  deviceCondition: string;
  diagnosisNotes: string;
  workPerformed: string;
  notes: string;
};

type Options = {
  repair?: Repair;
  mode: "create" | "edit";
  defaultCustomerId?: number;
  defaultDeviceId?: number;
};

function toValues(
  repair?: Repair,
  defaultCustomerId?: number,
  defaultDeviceId?: number,
): FormValues {
  return {
    customerId: String(repair?.customerId ?? defaultCustomerId ?? ""),
    deviceId: String(repair?.deviceId ?? defaultDeviceId ?? ""),
    status: repair?.status ?? "received",
    reportedProblem: repair?.reportedProblem ?? "",
    accessoriesReceived: repair?.accessoriesReceived ?? "",
    deviceCondition: repair?.deviceCondition ?? "",
    diagnosisNotes: repair?.diagnosisNotes ?? "",
    workPerformed: repair?.workPerformed ?? "",
    notes: repair?.notes ?? "",
  };
}

function toInput(value: FormValues): RepairInput {
  return {
    customerId: Number(value.customerId),
    deviceId: Number(value.deviceId),
    status: (value.status || undefined) as RepairStatus | undefined,
    reportedProblem: value.reportedProblem || null,
    accessoriesReceived: value.accessoriesReceived || null,
    deviceCondition: value.deviceCondition || null,
    diagnosisNotes: value.diagnosisNotes || null,
    workPerformed: value.workPerformed || null,
    notes: value.notes || null,
  };
}

export function useRepairForm({
  repair,
  mode,
  defaultCustomerId,
  defaultDeviceId,
}: Options) {
  const navigate = useNavigate();

  const form = useForm({
    defaultValues: toValues(repair, defaultCustomerId, defaultDeviceId),
    onSubmit: async ({ value }) => {
      const input = toInput(value);

      if (mode === "create") {
        if (!Number.isFinite(input.customerId) || input.customerId <= 0) {
          throw new Error("Customer is required");
        }
        if (!Number.isFinite(input.deviceId) || input.deviceId <= 0) {
          throw new Error("Device is required");
        }
        const created = await repairsApi.create(input);
        navigate(`/repairs/${created.id}`);
        return;
      }

      if (!repair) {
        throw new Error("Missing repair for edit");
      }
      const updated = await repairsApi.update(repair.id, input);
      navigate(`/repairs/${updated.id}`);
    },
  });

  return form;
}
