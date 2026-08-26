import { useForm } from "@tanstack/react-form";
import { useNavigate } from "react-router";
import { devicesApi } from "@/features/devices/api/devicesApi";
import type { Device, DeviceInput } from "@/features/devices/types/device";

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
  defaultCustomerId?: number;
};

function toValues(device?: Device, defaultCustomerId?: number): FormValues {
  return {
    customerId: String(device?.customerId ?? defaultCustomerId ?? ""),
    deviceType: device?.deviceType ?? "",
    manufacturer: device?.manufacturer ?? "",
    model: device?.model ?? "",
    serialNumber: device?.serialNumber ?? "",
    accessories: device?.accessories ?? "",
    notes: device?.notes ?? "",
  };
}

export function useDeviceForm({ device, mode, defaultCustomerId }: Options) {
  const navigate = useNavigate();

  const form = useForm({
    defaultValues: toValues(device, defaultCustomerId),
    onSubmit: async ({ value }) => {
      const customerId = Number(value.customerId);
      if (!Number.isFinite(customerId) || customerId <= 0) {
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
        navigate(`/devices/${created.id}`);
        return;
      }

      if (!device) {
        throw new Error("Missing device for edit");
      }
      const updated = await devicesApi.update(device.id, input);
      navigate(`/devices/${updated.id}`);
    },
  });

  return form;
}
