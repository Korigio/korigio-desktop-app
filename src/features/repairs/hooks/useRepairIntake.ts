import { useCallback, useEffect, useRef, useState } from "react";
import { useI18n } from "@/shared/hooks/useI18n";
import { useFlowWizard } from "@/shared/hooks/useFlowWizard";
import {
  customerLabel,
  useCustomerSearchCombobox,
} from "@/features/customers/hooks/useCustomerSearchCombobox";
import { useCustomerForm } from "@/features/customers/hooks/useCustomerForm";
import type { Customer } from "@/features/customers/types/customer";
import { useDeviceSearchCombobox } from "@/features/devices/hooks/useDeviceSearchCombobox";
import { useDeviceForm } from "@/features/devices/hooks/useDeviceForm";
import {
  deviceLabel,
  type Device,
} from "@/features/devices/types/device";
import { useRepairForm } from "@/features/repairs/hooks/useRepairForm";
import type { Repair } from "@/features/repairs/types/repair";

export const INTAKE_STEPS = [
  "customer",
  "device",
  "details",
  "review",
  "done",
] as const;

export type IntakeStep = (typeof INTAKE_STEPS)[number];

export function useRepairIntake() {
  const { t } = useI18n();

  const [customerCreateOpen, setCustomerCreateOpen] = useState(false);
  const [deviceCreateOpen, setDeviceCreateOpen] = useState(false);

  const [customer, setCustomer] = useState<Customer | null>(null);
  const [createdCustomerId, setCreatedCustomerId] = useState<number | null>(
    null,
  );
  const [device, setDevice] = useState<Device | null>(null);

  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [createdRepair, setCreatedRepair] = useState<Repair | null>(null);

  const createButtonRef = useRef<HTMLButtonElement>(null);

  const customerSearch = useCustomerSearchCombobox();
  const deviceSearch = useDeviceSearchCombobox({
    customerId: customer?.id,
  });

  const customerSearchFocus = customerSearch.focus;
  const deviceSearchFocus = deviceSearch.focus;
  const customerSearchReset = customerSearch.reset;
  const deviceSearchReset = deviceSearch.reset;

  const isNewCustomer =
    customer !== null && createdCustomerId === customer.id;
  const isNewCustomerRef = useRef(isNewCustomer);
  isNewCustomerRef.current = isNewCustomer;

  const clearError = useCallback(() => setError(null), []);

  const wizard = useFlowWizard<IntakeStep>({
    steps: INTAKE_STEPS,
    onEnter: {
      customer: () => {
        window.setTimeout(() => {
          customerSearchFocus();
        }, 0);
      },
      device: () => {
        window.setTimeout(() => {
          if (!isNewCustomerRef.current) {
            deviceSearchFocus();
          }
        }, 0);
      },
      review: () => {
        window.setTimeout(() => {
          createButtonRef.current?.focus();
        }, 0);
      },
    },
  });

  const { goTo, next, back, reset: resetWizardSteps, stepId } = wizard;

  const customerForm = useCustomerForm({
    mode: "create",
    navigateOnSuccess: false,
    onSuccess: (created) => {
      setCustomerCreateOpen(false);
      customerSearch.select(created);
      setCustomer(created);
      setCreatedCustomerId(created.id);
      setDevice(null);
      deviceSearchReset();
      setDeviceCreateOpen(false);
      goTo("device");
    },
  });

  const deviceForm = useDeviceForm({
    mode: "create",
    defaultCustomerId: customer?.id,
    navigateOnSuccess: false,
    onSuccess: (created) => {
      setDeviceCreateOpen(false);
      deviceSearch.select(created);
      setDevice(created);
      goTo("details");
    },
  });

  const repairForm = useRepairForm({
    mode: "create",
    defaultCustomerId: customer?.id,
    defaultDeviceId: device?.id,
    navigateOnSuccess: false,
    onSuccess: (created) => {
      setCreatedRepair(created);
      goTo("done");
    },
  });

  const customerFormReset = customerForm.reset;
  const deviceFormReset = deviceForm.reset;
  const repairFormReset = repairForm.reset;

  useEffect(() => {
    if (!customer?.id) {
      return;
    }
    deviceForm.setFieldValue("customerId", String(customer.id));
    repairForm.setFieldValue("customerId", String(customer.id));
  }, [customer?.id]); // eslint-disable-line react-hooks/exhaustive-deps -- sync ids only

  useEffect(() => {
    if (!device?.id) {
      return;
    }
    repairForm.setFieldValue("deviceId", String(device.id));
  }, [device?.id]); // eslint-disable-line react-hooks/exhaustive-deps -- sync ids only

  const resetWizard = useCallback(() => {
    setCustomerCreateOpen(false);
    setDeviceCreateOpen(false);
    setCustomer(null);
    setCreatedCustomerId(null);
    setDevice(null);
    setSubmitting(false);
    setError(null);
    setCreatedRepair(null);
    customerSearchReset();
    deviceSearchReset();
    customerFormReset();
    deviceFormReset();
    repairFormReset();
    resetWizardSteps();
  }, [
    customerFormReset,
    customerSearchReset,
    deviceFormReset,
    deviceSearchReset,
    repairFormReset,
    resetWizardSteps,
  ]);

  const goBack = useCallback(() => {
    clearError();
    if (stepId === "device") {
      setDeviceCreateOpen(false);
      setDevice(null);
      deviceSearchReset();
      deviceFormReset();
      goTo("customer");
      return;
    }
    if (stepId === "details" || stepId === "review") {
      back();
    }
  }, [
    back,
    clearError,
    deviceFormReset,
    deviceSearchReset,
    goTo,
    stepId,
  ]);

  const openCustomerCreate = useCallback(() => {
    clearError();
    customerFormReset();
    customerForm.setFieldValue("name", customerSearch.query.trim());
    setCustomerCreateOpen(true);
  }, [clearError, customerForm, customerFormReset, customerSearch.query]);

  const onCustomerCreateOpenChange = useCallback((open: boolean) => {
    setCustomerCreateOpen(open);
  }, []);

  const openDeviceCreate = useCallback(() => {
    if (!customer) {
      return;
    }
    clearError();
    deviceFormReset();
    deviceForm.setFieldValue("customerId", String(customer.id));
    deviceForm.setFieldValue("serialNumber", deviceSearch.query.trim());
    setDeviceCreateOpen(true);
  }, [
    clearError,
    customer,
    deviceForm,
    deviceFormReset,
    deviceSearch.query,
  ]);

  const onDeviceCreateOpenChange = useCallback((open: boolean) => {
    setDeviceCreateOpen(open);
  }, []);

  const continueFromCustomer = useCallback(async () => {
    clearError();

    const selected = customerSearch.selected;
    if (!selected) {
      setError(t("repairs.validation.customerRequired"));
      customerSearchFocus();
      return;
    }
    setCustomer(selected);
    setCreatedCustomerId(null);
    setDevice(null);
    deviceSearchReset();
    deviceFormReset();
    setDeviceCreateOpen(false);
    goTo("device");
  }, [
    clearError,
    customerSearch.selected,
    customerSearchFocus,
    deviceFormReset,
    deviceSearchReset,
    goTo,
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
      } catch (err) {
        setError(err instanceof Error ? err.message : "Failed to create device");
      } finally {
        setSubmitting(false);
      }
      return;
    }

    const selected = deviceSearch.selected;
    if (!selected) {
      setError(t("repairs.validation.deviceRequired"));
      deviceSearchFocus();
      return;
    }
    setDevice(selected);
    goTo("details");
  }, [
    clearError,
    customer,
    deviceForm,
    deviceSearch.selected,
    deviceSearchFocus,
    goTo,
    isNewCustomer,
    t,
  ]);

  const continueFromDetails = useCallback(() => {
    clearError();
    const values = repairForm.state.values;
    if (!values.reportedProblem.trim()) {
      setError(t("repairs.intake.problemRequired"));
      return;
    }
    if (customer?.id) {
      repairForm.setFieldValue("customerId", String(customer.id));
    }
    if (device?.id) {
      repairForm.setFieldValue("deviceId", String(device.id));
    }
    repairForm.setFieldValue("status", "received");
    next();
  }, [clearError, customer?.id, device?.id, next, repairForm, t]);

  const createRepair = useCallback(async () => {
    clearError();

    if (!customer) {
      setError(t("repairs.validation.customerRequired"));
      goTo("customer");
      return;
    }
    if (!device) {
      setError(t("repairs.validation.deviceRequired"));
      goTo("device");
      return;
    }

    repairForm.setFieldValue("customerId", String(customer.id));
    repairForm.setFieldValue("deviceId", String(device.id));
    repairForm.setFieldValue("status", "received");

    setSubmitting(true);
    try {
      await repairForm.handleSubmit();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to create repair");
    } finally {
      setSubmitting(false);
    }
  }, [clearError, customer, device, goTo, repairForm, t]);

  const continuePrimary = useCallback(async () => {
    if (stepId === "customer") {
      await continueFromCustomer();
      return;
    }
    if (stepId === "device") {
      await continueFromDevice();
      return;
    }
    if (stepId === "details") {
      continueFromDetails();
      return;
    }
    if (stepId === "review") {
      await createRepair();
    }
  }, [
    continueFromCustomer,
    continueFromDetails,
    continueFromDevice,
    createRepair,
    stepId,
  ]);

  const details = repairForm.state.values;

  const reviewItems = [
    {
      label: t("repairs.fields.customer"),
      value: customer ? customerLabel(customer) : "—",
    },
    {
      label: t("repairs.fields.device"),
      value: device ? deviceLabel(device) : "—",
    },
    {
      label: t("repairs.fields.reportedProblem"),
      value: details.reportedProblem.trim() || "—",
    },
    {
      label: t("repairs.fields.accessoriesReceived"),
      value: details.accessoriesReceived.trim() || "—",
    },
    {
      label: t("repairs.fields.deviceCondition"),
      value: details.deviceCondition.trim() || "—",
    },
    {
      label: t("repairs.fields.expectedPickupAt"),
      value: details.expectedPickupAt.trim() || "—",
    },
    {
      label: t("repairs.fields.notes"),
      value: details.notes.trim() || "—",
    },
  ];

  return {
    step: stepId,
    customerCreateOpen,
    deviceCreateOpen,
    isNewCustomer,
    customer,
    device,
    customerSearch,
    deviceSearch,
    customerForm,
    deviceForm,
    repairForm,
    createButtonRef,
    submitting,
    error,
    createdRepair,
    reviewItems,
    openCustomerCreate,
    onCustomerCreateOpenChange,
    openDeviceCreate,
    onDeviceCreateOpenChange,
    goBack,
    continuePrimary,
    createRepair,
    resetWizard,
    startAnother: resetWizard,
  };
}

export type RepairIntakeState = ReturnType<typeof useRepairIntake>;
