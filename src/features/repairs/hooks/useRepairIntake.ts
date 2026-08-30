import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { useI18n } from "@/shared/hooks/useI18n";
import { useFlowWizard } from "@/shared/hooks/useFlowWizard";
import { companiesApi } from "@/features/companies/api/companiesApi";
import {
  companyLabel,
  type Company,
} from "@/features/companies/types/company";
import { customersApi } from "@/features/customers/api/customersApi";
import {
  customerLabel,
  useCustomerSearchCombobox,
} from "@/features/customers/hooks/useCustomerSearchCombobox";
import { useCustomerForm } from "@/features/customers/hooks/useCustomerForm";
import type { Customer } from "@/features/customers/types/customer";
import { devicesApi } from "@/features/devices/api/devicesApi";
import { useDeviceSearchCombobox } from "@/features/devices/hooks/useDeviceSearchCombobox";
import { useDeviceForm } from "@/features/devices/hooks/useDeviceForm";
import {
  deviceLabel,
  type Device,
} from "@/features/devices/types/device";
import { useRepairForm } from "@/features/repairs/hooks/useRepairForm";
import type { Repair } from "@/features/repairs/types/repair";

const BASE_INTAKE_STEPS = [
  "customer",
  "device",
  "details",
  "review",
  "done",
] as const;

export type IntakeStep =
  | "company"
  | (typeof BASE_INTAKE_STEPS)[number];

export function buildIntakeSteps(includeCompany: boolean): readonly IntakeStep[] {
  return includeCompany
    ? (["company", ...BASE_INTAKE_STEPS] as const)
    : BASE_INTAKE_STEPS;
}

/** @deprecated Prefer `buildIntakeSteps` / `intake.steps` for dynamic company step. */
export const INTAKE_STEPS = BASE_INTAKE_STEPS;

export type IntakeGate = "loading" | "no-company" | "ready";

export type RepairIntakePresets = {
  presetCustomerId?: string;
  presetDeviceId?: string;
};

function stepAfterKnownEntities(
  customer: Customer | null,
  device: Device | null,
): Extract<IntakeStep, "customer" | "device" | "details"> {
  if (customer && device) {
    return "details";
  }
  if (customer) {
    return "device";
  }
  return "customer";
}

export function useRepairIntake(presets: RepairIntakePresets = {}) {
  const { t } = useI18n();
  const { presetCustomerId, presetDeviceId } = presets;

  const [gate, setGate] = useState<IntakeGate>("loading");
  const [presetsReady, setPresetsReady] = useState(() => !presetCustomerId);
  const appliedPresetsRef = useRef(false);
  const [companies, setCompanies] = useState<Company[]>([]);
  const [company, setCompany] = useState<Company | null>(null);
  const [companiesError, setCompaniesError] = useState<string | null>(null);

  const [customerCreateOpen, setCustomerCreateOpen] = useState(false);
  const [deviceCreateOpen, setDeviceCreateOpen] = useState(false);

  const [customer, setCustomer] = useState<Customer | null>(null);
  const [createdCustomerId, setCreatedCustomerId] = useState<string | null>(
    null,
  );
  const [device, setDevice] = useState<Device | null>(null);

  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [createdRepair, setCreatedRepair] = useState<Repair | null>(null);

  const createButtonRef = useRef<HTMLButtonElement>(null);

  const includeCompanyStep = companies.length > 1;
  const steps = useMemo(
    () => buildIntakeSteps(includeCompanyStep),
    [includeCompanyStep],
  );

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
    steps,
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

  const goToRef = useRef(goTo);
  goToRef.current = goTo;
  const includeCompanyStepRef = useRef(includeCompanyStep);
  includeCompanyStepRef.current = includeCompanyStep;
  const customerSearchSelectRef = useRef(customerSearch.select);
  customerSearchSelectRef.current = customerSearch.select;
  const deviceSearchSelectRef = useRef(deviceSearch.select);
  deviceSearchSelectRef.current = deviceSearch.select;

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
    defaultCompanyId: company?.id,
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
    let cancelled = false;
    setGate("loading");
    setCompaniesError(null);

    void companiesApi
      .list({ includeArchived: false, page: 1, pageSize: 100 })
      .then((result) => {
        if (cancelled) {
          return;
        }
        const items = result.items;
        setCompanies(items);
        if (items.length === 0) {
          setCompany(null);
          setGate("no-company");
          return;
        }
        if (items.length === 1) {
          setCompany(items[0] ?? null);
        } else {
          const preferred =
            items.find((item) => item.isDefault) ?? items[0] ?? null;
          setCompany(preferred);
        }
        setGate("ready");
      })
      .catch((err: unknown) => {
        if (cancelled) {
          return;
        }
        setCompanies([]);
        setCompany(null);
        setCompaniesError(
          err instanceof Error ? err.message : "Failed to load companies",
        );
        setGate("no-company");
      });

    return () => {
      cancelled = true;
    };
  }, []);

  useEffect(() => {
    if (!company?.id) {
      return;
    }
    repairForm.setFieldValue("companyId", String(company.id));
  }, [company?.id]); // eslint-disable-line react-hooks/exhaustive-deps -- sync id only

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

  const selectCompany = useCallback((next: Company) => {
    setCompany(next);
    clearError();
  }, [clearError]);

  const applyPresetsAndSkip = useCallback(async () => {
    let appliedCustomer: Customer | null = null;
    let appliedDevice: Device | null = null;

    if (presetCustomerId) {
      try {
        const fetched = await customersApi.get(presetCustomerId);
        appliedCustomer = fetched;
        setCustomer(fetched);
        setCreatedCustomerId(null);
        customerSearchSelectRef.current(fetched);
      } catch {
        appliedCustomer = null;
      }

      if (appliedCustomer && presetDeviceId) {
        try {
          const fetched = await devicesApi.get(presetDeviceId);
          if (fetched.customerId === appliedCustomer.id) {
            appliedDevice = fetched;
            setDevice(fetched);
            deviceSearchSelectRef.current(fetched);
          }
        } catch {
          // Ignore invalid / mismatched device presets.
        }
      }
    }

    if (includeCompanyStepRef.current) {
      goToRef.current("company");
    } else {
      goToRef.current(stepAfterKnownEntities(appliedCustomer, appliedDevice));
    }
  }, [presetCustomerId, presetDeviceId]);

  useEffect(() => {
    if (gate !== "ready" || appliedPresetsRef.current) {
      return;
    }
    if (!presetCustomerId) {
      appliedPresetsRef.current = true;
      setPresetsReady(true);
      return;
    }

    appliedPresetsRef.current = true;
    let cancelled = false;
    void applyPresetsAndSkip().finally(() => {
      if (!cancelled) {
        setPresetsReady(true);
      }
    });
    return () => {
      cancelled = true;
    };
  }, [applyPresetsAndSkip, gate, presetCustomerId]);

  const resetWizard = useCallback(() => {
    if (presetCustomerId) {
      setPresetsReady(false);
    }
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
    if (companies.length === 1) {
      setCompany(companies[0] ?? null);
    } else if (companies.length > 1) {
      const preferred =
        companies.find((item) => item.isDefault) ?? companies[0] ?? null;
      setCompany(preferred);
    }
    resetWizardSteps();
    if (!presetCustomerId) {
      return;
    }
    void applyPresetsAndSkip().finally(() => {
      setPresetsReady(true);
    });
  }, [
    applyPresetsAndSkip,
    companies,
    customerFormReset,
    customerSearchReset,
    deviceFormReset,
    deviceSearchReset,
    presetCustomerId,
    repairFormReset,
    resetWizardSteps,
  ]);

  const firstStep: IntakeStep = includeCompanyStep ? "company" : "customer";

  const goBack = useCallback(() => {
    clearError();
    if (stepId === "customer") {
      if (includeCompanyStep) {
        goTo("company");
      }
      return;
    }
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
    includeCompanyStep,
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

  const continueFromCompany = useCallback(() => {
    clearError();
    if (!company) {
      setError(t("repairs.validation.companyRequired"));
      return;
    }
    repairForm.setFieldValue("companyId", String(company.id));
    goTo(stepAfterKnownEntities(customer, device));
  }, [clearError, company, customer, device, goTo, repairForm, t]);

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
    if (company?.id) {
      repairForm.setFieldValue("companyId", String(company.id));
    }
    if (customer?.id) {
      repairForm.setFieldValue("customerId", String(customer.id));
    }
    if (device?.id) {
      repairForm.setFieldValue("deviceId", String(device.id));
    }
    repairForm.setFieldValue("status", "received");
    next();
  }, [clearError, company?.id, customer?.id, device?.id, next, repairForm, t]);

  const createRepair = useCallback(async () => {
    clearError();

    if (!company) {
      setError(t("repairs.validation.companyRequired"));
      goTo(includeCompanyStep ? "company" : firstStep);
      return;
    }
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

    repairForm.setFieldValue("companyId", String(company.id));
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
  }, [
    clearError,
    company,
    customer,
    device,
    firstStep,
    goTo,
    includeCompanyStep,
    repairForm,
    t,
  ]);

  const continuePrimary = useCallback(async () => {
    if (stepId === "company") {
      continueFromCompany();
      return;
    }
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
    continueFromCompany,
    continueFromCustomer,
    continueFromDetails,
    continueFromDevice,
    createRepair,
    stepId,
  ]);

  const details = repairForm.state.values;

  const reviewItems = [
    {
      label: t("repairs.fields.company"),
      value: company ? companyLabel(company) : "—",
    },
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

  const formGate: IntakeGate =
    gate === "ready" && !presetsReady ? "loading" : gate;

  return {
    gate: formGate,
    companies,
    companiesError,
    company,
    selectCompany,
    includeCompanyStep,
    steps,
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
