import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { useI18n } from "@/shared/hooks/useI18n";
import { useFlowWizard } from "@/shared/hooks/useFlowWizard";
import { useCustomerSearchCombobox } from "@/features/customers/hooks/useCustomerSearchCombobox";
import { useCustomerForm } from "@/features/customers/hooks/useCustomerForm";
import type { Customer } from "@/features/customers/types/customer";
import { useDeviceSearchCombobox } from "@/features/devices/hooks/useDeviceSearchCombobox";
import { useDeviceForm } from "@/features/devices/hooks/useDeviceForm";
import type { Device } from "@/features/devices/types/device";
import { useRepairForm } from "@/features/repairs/hooks/useRepairForm";
import { useRepairIntakeCompanies } from "@/features/repairs/hooks/useRepairIntakeCompanies";
import { useRepairIntakeEntityActions } from "@/features/repairs/hooks/useRepairIntakeEntityActions";
import { useRepairIntakePresets } from "@/features/repairs/hooks/useRepairIntakePresets";
import type { Repair } from "@/features/repairs/types/repair";
import {
  BASE_INTAKE_STEPS,
  type IntakeGate,
  type IntakeStep,
  type RepairIntakePresets,
} from "@/features/repairs/types/repairIntake";
import {
  buildIntakeSteps,
  buildRepairIntakeReviewItems,
  resolveIntakeEstimateFields,
  stepAfterKnownEntities,
} from "@/features/repairs/utils/repairIntake";
import { settingsApi } from "@/features/settings/api/settingsApi";
import type { ShopSettings } from "@/features/settings/types/shopSettings";

export type {
  IntakeGate,
  IntakeStep,
  RepairIntakePresets,
} from "@/features/repairs/types/repairIntake";
export { buildIntakeSteps } from "@/features/repairs/utils/repairIntake";

/** @deprecated Prefer `buildIntakeSteps` / `intake.steps` for dynamic company step. */
export const INTAKE_STEPS = BASE_INTAKE_STEPS;

export function useRepairIntake(presets: RepairIntakePresets = {}) {
  const { t } = useI18n();
  const { presetCustomerId, presetDeviceId } = presets;

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
  const [shopSettings, setShopSettings] = useState<ShopSettings | null>(null);

  const createButtonRef = useRef<HTMLButtonElement>(null);
  const clearError = useCallback(() => setError(null), []);
  const {
    gate,
    companies,
    companiesError,
    company,
    selectCompany,
    resetCompany,
  } = useRepairIntakeCompanies(clearError);

  useEffect(() => {
    let cancelled = false;
    void settingsApi
      .getShopSettings()
      .then((settings) => {
        if (!cancelled) setShopSettings(settings);
      })
      .catch(() => {
        /* review/preview can proceed without tax formatting */
      });
    return () => {
      cancelled = true;
    };
  }, []);

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

  const isNewCustomer = customer !== null && createdCustomerId === customer.id;
  const isNewCustomerRef = useRef(isNewCustomer);
  isNewCustomerRef.current = isNewCustomer;

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

  const { presetsReady, reapplyPresets } = useRepairIntakePresets({
    presetCustomerId,
    presetDeviceId,
    gate,
    includeCompanyStep,
    goTo,
    selectCustomer: customerSearch.select,
    selectDevice: deviceSearch.select,
    setCustomer,
    setCreatedCustomerId,
    setDevice,
  });

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
    resetCompany();
    resetWizardSteps();
    reapplyPresets();
  }, [
    customerFormReset,
    customerSearchReset,
    deviceFormReset,
    deviceSearchReset,
    reapplyPresets,
    repairFormReset,
    resetCompany,
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

  const {
    openCustomerCreate,
    openDeviceCreate,
    continueFromCustomer,
    continueFromDevice,
  } = useRepairIntakeEntityActions({
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
  });

  const onCustomerCreateOpenChange = useCallback((open: boolean) => {
    setCustomerCreateOpen(open);
  }, []);

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

  const mapEstimateError = useCallback(
    (reason: "discountNeedsPrice" | "invalidPrice" | "invalidDiscount") => {
      if (reason === "discountNeedsPrice") {
        return t("repairs.intake.estimate.validation.discountNeedsPrice");
      }
      if (reason === "invalidDiscount") {
        return t("repairs.diagnosisFlow.estimatePreviewInvalid");
      }
      return t("repairs.diagnosisFlow.errors.estimateInvalid");
    },
    [t],
  );

  const continueFromDetails = useCallback(() => {
    clearError();
    const values = repairForm.state.values;
    if (!values.reportedProblem.trim()) {
      setError(t("repairs.intake.problemRequired"));
      return;
    }
    const estimate = resolveIntakeEstimateFields(
      values.estimateMajor,
      values.estimateDiscountPercent,
    );
    if (!estimate.ok) {
      setError(mapEstimateError(estimate.reason));
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
  }, [
    clearError,
    company?.id,
    customer?.id,
    device?.id,
    mapEstimateError,
    next,
    repairForm,
    t,
  ]);

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

    const estimate = resolveIntakeEstimateFields(
      repairForm.state.values.estimateMajor,
      repairForm.state.values.estimateDiscountPercent,
    );
    if (!estimate.ok) {
      setError(mapEstimateError(estimate.reason));
      goTo("details");
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
    mapEstimateError,
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

  const reviewItems = buildRepairIntakeReviewItems(
    t,
    company,
    customer,
    device,
    details,
    shopSettings,
  );

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
    shopSettings,
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
