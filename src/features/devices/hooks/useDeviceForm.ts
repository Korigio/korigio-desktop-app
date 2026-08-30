import { useForm } from "@tanstack/react-form";
import { useRef } from "react";
import { useNavigate } from "react-router";
import { devicesApi } from "@/features/devices/api/devicesApi";
import type { Device, DeviceInput } from "@/features/devices/types/device";
import { isEntityId } from "@/shared/utils/entityId";

type FormValues = {
  customerId: string;
  deviceType: string;
  manufacturer: string;
  model: string;
  serialNumber: string;
  accessories: string;
  notes: string;
};

type Options = {
  device?: Device;
  mode: "create" | "edit";
  defaultCustomerId?: string;
  onSuccess?: (entity: Device) => void;
  navigateOnSuccess?: boolean;
};

function toValues(device?: Device, defaultCustomerId?: string): FormValues {
  return {
    customerId: device?.customerId ?? defaultCustomerId ?? "",
    deviceType: device?.deviceType ?? "",
    manufacturer: device?.manufacturer ?? "",
    model: device?.model ?? "",
    serialNumber: device?.serialNumber ?? "",
    accessories: device?.accessories ?? "",
    notes: device?.notes ?? "",
  };
}

export function useDeviceForm({
  device,
  mode,
  defaultCustomerId,
  onSuccess,
  navigateOnSuccess = true,
}: Options) {
  const navigate = useNavigate();
  const onSuccessRef = useRef(onSuccess);
  onSuccessRef.current = onSuccess;
  const navigateOnSuccessRef = useRef(navigateOnSuccess);
  navigateOnSuccessRef.current = navigateOnSuccess;

  const form = useForm({
    defaultValues: toValues(device, defaultCustomerId),
    onSubmit: async ({ value }) => {
      const customerId = value.customerId;
      if (!isEntityId(customerId)) {
        throw new Error("Customer is required");
      }

      const input: DeviceInput = {
        customerId,
        deviceType: value.deviceType || null,
        manufacturer: value.manufacturer || null,
        model: value.model || null,
        serialNumber: value.serialNumber || null,
        accessories: value.accessories || null,
        notes: value.notes || null,
      };

      if (mode === "create") {
        const created = await devicesApi.create(input);
        onSuccessRef.current?.(created);
        if (navigateOnSuccessRef.current) {
          navigate(`/devices/${created.id}`);
        }
        return;
      }

      if (!device) {
        throw new Error("Missing device for edit");
      }
      const updated = await devicesApi.update(device.id, input);
      onSuccessRef.current?.(updated);
      if (navigateOnSuccessRef.current) {
        navigate(`/devices/${updated.id}`);
      }
    },
  });

  return form;
}
