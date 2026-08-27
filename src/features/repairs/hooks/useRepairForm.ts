import { useForm } from "@tanstack/react-form";
import { useRef } from "react";
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
  expectedPickupAt: string;
  diagnosisNotes: string;
  workPerformed: string;
  notes: string;
};

type Options = {
  repair?: Repair;
  mode: "create" | "edit";
  defaultCustomerId?: number;
  defaultDeviceId?: number;
  onSuccess?: (entity: Repair) => void;
  navigateOnSuccess?: boolean;
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
    expectedPickupAt: repair?.expectedPickupAt ?? "",
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
    expectedPickupAt: value.expectedPickupAt || null,
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
  onSuccess,
  navigateOnSuccess = true,
}: Options) {
  const navigate = useNavigate();
  const onSuccessRef = useRef(onSuccess);
  onSuccessRef.current = onSuccess;
  const navigateOnSuccessRef = useRef(navigateOnSuccess);
  navigateOnSuccessRef.current = navigateOnSuccess;

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
        onSuccessRef.current?.(created);
        if (navigateOnSuccessRef.current) {
          navigate(`/repairs/${created.id}`);
        }
        return;
      }

      if (!repair) {
        throw new Error("Missing repair for edit");
      }
      const updated = await repairsApi.update(repair.id, input);
      onSuccessRef.current?.(updated);
      if (navigateOnSuccessRef.current) {
        navigate(`/repairs/${updated.id}`);
      }
    },
  });

  return form;
}
