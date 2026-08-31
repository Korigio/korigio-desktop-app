import { useForm } from "@tanstack/react-form";
import { useRef } from "react";
import { useNavigate } from "react-router";
import { repairsApi } from "@/features/repairs/api/repairsApi";
import type {
  Repair,
  RepairInput,
  RepairStatus,
} from "@/features/repairs/types/repair";
import { isEntityId } from "@/shared/utils/entityId";

type FormValues = {
  customerId: string;
  deviceId: string;
  companyId: string;
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
  defaultCustomerId?: string;
  defaultDeviceId?: string;
  defaultCompanyId?: string;
  onSuccess?: (entity: Repair) => void;
  navigateOnSuccess?: boolean;
};

function toValues(
  defaultCustomerId?: string,
  defaultDeviceId?: string,
  defaultCompanyId?: string,
): FormValues {
  return {
    customerId: defaultCustomerId ?? "",
    deviceId: defaultDeviceId ?? "",
    companyId: defaultCompanyId ?? "",
    status: "received",
    reportedProblem: "",
    accessoriesReceived: "",
    deviceCondition: "",
    expectedPickupAt: "",
    diagnosisNotes: "",
    workPerformed: "",
    notes: "",
  };
}

function toInput(value: FormValues): RepairInput {
  return {
    customerId: value.customerId,
    deviceId: value.deviceId,
    companyId: value.companyId,
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
  defaultCustomerId,
  defaultDeviceId,
  defaultCompanyId,
  onSuccess,
  navigateOnSuccess = true,
}: Options) {
  const navigate = useNavigate();
  const onSuccessRef = useRef(onSuccess);
  onSuccessRef.current = onSuccess;
  const navigateOnSuccessRef = useRef(navigateOnSuccess);
  navigateOnSuccessRef.current = navigateOnSuccess;

  const form = useForm({
    defaultValues: toValues(
      defaultCustomerId,
      defaultDeviceId,
      defaultCompanyId,
    ),
    onSubmit: async ({ value }) => {
      const input = toInput(value);

      if (!isEntityId(input.customerId)) {
        throw new Error("Customer is required");
      }
      if (!isEntityId(input.deviceId)) {
        throw new Error("Device is required");
      }
      if (!isEntityId(input.companyId)) {
        throw new Error("Company is required");
      }
      const created = await repairsApi.create(input);
      onSuccessRef.current?.(created);
      if (navigateOnSuccessRef.current) {
        navigate(`/repairs/${created.id}`);
      }
    },
  });

  return form;
}
