import { useCallback, type Dispatch, type SetStateAction } from "react";
import type { useCustomerSearchCombobox } from "@/features/customers/hooks/useCustomerSearchCombobox";
import type { useCustomerForm } from "@/features/customers/hooks/useCustomerForm";
import type { Customer } from "@/features/customers/types/customer";
import type { useDeviceSearchCombobox } from "@/features/devices/hooks/useDeviceSearchCombobox";
import type { useDeviceForm } from "@/features/devices/hooks/useDeviceForm";
import type { Device } from "@/features/devices/types/device";
import type { IntakeStep } from "@/features/repairs/types/repairIntake";

type Params = {
  t: (key: string) => string;
  customer: Customer | null;
  isNewCustomer: boolean;
  customerSearch: ReturnType<typeof useCustomerSearchCombobox>;
  deviceSearch: ReturnType<typeof useDeviceSearchCombobox>;
  customerForm: ReturnType<typeof useCustomerForm>;
  deviceForm: ReturnType<typeof useDeviceForm>;
  clearError: () => void;
  goTo: (step: IntakeStep) => void;
  setCustomer: Dispatch<SetStateAction<Customer | null>>;
  setCreatedCustomerId: Dispatch<SetStateAction<string | null>>;
  setDevice: Dispatch<SetStateAction<Device | null>>;
  setCustomerCreateOpen: Dispatch<SetStateAction<boolean>>;
  setDeviceCreateOpen: Dispatch<SetStateAction<boolean>>;
  setSubmitting: Dispatch<SetStateAction<boolean>>;
  setError: Dispatch<SetStateAction<string | null>>;
};

export function useRepairIntakeEntityActions({
  t,
  customer,
  isNewCustomer,
  customerSearch,
  deviceSearch,
  customerForm,
  deviceForm,
  clearError,
  goTo,
  setCustomer,
  setCreatedCustomerId,
  setDevice,
  setCustomerCreateOpen,
  setDeviceCreateOpen,
  setSubmitting,
  setError,
}: Params) {
  const openCustomerCreate = useCallback(() => {
    clearError();
    customerForm.reset();
    customerForm.setFieldValue("name", customerSearch.query.trim());
    setCustomerCreateOpen(true);
  }, [clearError, customerForm, customerSearch.query, setCustomerCreateOpen]);

  const openDeviceCreate = useCallback(() => {
    if (!customer) return;
    clearError();
    deviceForm.reset();
    deviceForm.setFieldValue("customerId", String(customer.id));
    deviceForm.setFieldValue("serialNumber", deviceSearch.query.trim());
    setDeviceCreateOpen(true);
  }, [
    clearError,
    customer,
    deviceForm,
    deviceSearch.query,
    setDeviceCreateOpen,
  ]);

  const continueFromCustomer = useCallback(async () => {
    clearError();
    const selected = customerSearch.selected;
    if (!selected) {
      setError(t("repairs.validation.customerRequired"));
      customerSearch.focus();
      return;
    }
    setCustomer(selected);
    setCreatedCustomerId(null);
    setDevice(null);
    deviceSearch.reset();
    deviceForm.reset();
    setDeviceCreateOpen(false);
    goTo("device");
  }, [
    clearError,
    customerSearch,
    deviceForm,
    deviceSearch,
    goTo,
    setCreatedCustomerId,
    setCustomer,
    setDevice,
    setDeviceCreateOpen,
    setError,
    t,
  ]);

  const continueFromDevice = useCallback(async () => {
    clearError();
    if (!customer) {
      setError(t("repairs.validation.customerRequired"));
      goTo("customer");
      return;
    }
    if (isNewCustomer) {
      deviceForm.setFieldValue("customerId", String(customer.id));
      setSubmitting(true);
      try {
        await deviceForm.handleSubmit();
      } catch (error) {
        setError(
          error instanceof Error ? error.message : "Failed to create device",
        );
      } finally {
        setSubmitting(false);
      }
      return;
    }
    const selected = deviceSearch.selected;
    if (!selected) {
      setError(t("repairs.validation.deviceRequired"));
      deviceSearch.focus();
      return;
    }
    setDevice(selected);
    goTo("details");
  }, [
    clearError,
    customer,
    deviceForm,
    deviceSearch,
    goTo,
    isNewCustomer,
    setDevice,
    setError,
    setSubmitting,
    t,
  ]);

  return {
    openCustomerCreate,
    openDeviceCreate,
    continueFromCustomer,
    continueFromDevice,
  };
}
