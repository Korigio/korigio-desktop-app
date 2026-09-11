import {
  useCallback,
  useEffect,
  useRef,
  useState,
  type Dispatch,
  type SetStateAction,
} from "react";
import { customersApi } from "@/features/customers/api/customersApi";
import type { Customer } from "@/features/customers/types/customer";
import { devicesApi } from "@/features/devices/api/devicesApi";
import type { Device } from "@/features/devices/types/device";
import type {
  IntakeGate,
  IntakeStep,
  RepairIntakePresets,
} from "@/features/repairs/types/repairIntake";
import { stepAfterKnownEntities } from "@/features/repairs/utils/repairIntake";

type Params = RepairIntakePresets & {
  gate: IntakeGate;
  includeCompanyStep: boolean;
  goTo: (step: IntakeStep) => void;
  selectCustomer: (customer: Customer) => void;
  selectDevice: (device: Device) => void;
  setCustomer: Dispatch<SetStateAction<Customer | null>>;
  setCreatedCustomerId: Dispatch<SetStateAction<string | null>>;
  setDevice: Dispatch<SetStateAction<Device | null>>;
};

export function useRepairIntakePresets({
  presetCustomerId,
  presetDeviceId,
  gate,
  includeCompanyStep,
  goTo,
  selectCustomer,
  selectDevice,
  setCustomer,
  setCreatedCustomerId,
  setDevice,
}: Params) {
  const [ready, setReady] = useState(() => !presetCustomerId);
  const appliedRef = useRef(false);
  const goToRef = useRef(goTo);
  goToRef.current = goTo;
  const includeCompanyStepRef = useRef(includeCompanyStep);
  includeCompanyStepRef.current = includeCompanyStep;
  const selectCustomerRef = useRef(selectCustomer);
  selectCustomerRef.current = selectCustomer;
  const selectDeviceRef = useRef(selectDevice);
  selectDeviceRef.current = selectDevice;

  const apply = useCallback(async () => {
    let appliedCustomer: Customer | null = null;
    let appliedDevice: Device | null = null;
    if (presetCustomerId) {
      try {
        appliedCustomer = await customersApi.get(presetCustomerId);
        setCustomer(appliedCustomer);
        setCreatedCustomerId(null);
        selectCustomerRef.current(appliedCustomer);
      } catch {
        appliedCustomer = null;
      }
      if (appliedCustomer && presetDeviceId) {
        try {
          const fetched = await devicesApi.get(presetDeviceId);
          if (fetched.customerId === appliedCustomer.id) {
            appliedDevice = fetched;
            setDevice(fetched);
            selectDeviceRef.current(fetched);
          }
        } catch {
          // Invalid and mismatched device presets intentionally fall back to selection.
        }
      }
    }
    goToRef.current(
      includeCompanyStepRef.current
        ? "company"
        : stepAfterKnownEntities(appliedCustomer, appliedDevice),
    );
  }, [
    presetCustomerId,
    presetDeviceId,
    setCreatedCustomerId,
    setCustomer,
    setDevice,
  ]);

  useEffect(() => {
    if (gate !== "ready" || appliedRef.current) return;
    appliedRef.current = true;
    if (!presetCustomerId) {
      setReady(true);
      return;
    }
    let cancelled = false;
    void apply().finally(() => {
      if (!cancelled) setReady(true);
    });
    return () => {
      cancelled = true;
    };
  }, [apply, gate, presetCustomerId]);

  const reapply = useCallback(() => {
    if (!presetCustomerId) return;
    setReady(false);
    void apply().finally(() => setReady(true));
  }, [apply, presetCustomerId]);

  return { presetsReady: ready, reapplyPresets: reapply };
}
