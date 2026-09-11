import { act, renderHook } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import type { Customer } from "@/features/customers/types/customer";
import { useRepairIntakeEntityActions } from "./useRepairIntakeEntityActions";

function params(overrides: Record<string, unknown> = {}) {
  return {
    t: (key: string) => key,
    customer: null,
    isNewCustomer: false,
    customerSearch: {
      selected: null,
      query: "",
      focus: vi.fn(),
      reset: vi.fn(),
    },
    deviceSearch: { selected: null, query: "", focus: vi.fn(), reset: vi.fn() },
    customerForm: { reset: vi.fn(), setFieldValue: vi.fn() },
    deviceForm: {
      reset: vi.fn(),
      setFieldValue: vi.fn(),
      handleSubmit: vi.fn(),
    },
    clearError: vi.fn(),
    goTo: vi.fn(),
    setCustomer: vi.fn(),
    setCreatedCustomerId: vi.fn(),
    setDevice: vi.fn(),
    setCustomerCreateOpen: vi.fn(),
    setDeviceCreateOpen: vi.fn(),
    setSubmitting: vi.fn(),
    setError: vi.fn(),
    ...overrides,
  } as unknown as Parameters<typeof useRepairIntakeEntityActions>[0];
}

describe("repair intake entity actions", () => {
  it("keeps focus on the missing customer boundary", async () => {
    const values = params();
    const { result } = renderHook(() => useRepairIntakeEntityActions(values));
    await act(() => result.current.continueFromCustomer());
    expect(values.setError).toHaveBeenCalledWith(
      "repairs.validation.customerRequired",
    );
    expect(values.customerSearch.focus).toHaveBeenCalledOnce();
    expect(values.goTo).not.toHaveBeenCalled();
  });

  it("surfaces device creation failures and always clears submitting", async () => {
    const failure = new Error("device save failed");
    const deviceForm = {
      reset: vi.fn(),
      setFieldValue: vi.fn(),
      handleSubmit: vi.fn().mockRejectedValue(failure),
    };
    const values = params({
      customer: { id: "customer" } as Customer,
      isNewCustomer: true,
      deviceForm,
    });
    const { result } = renderHook(() => useRepairIntakeEntityActions(values));
    await act(() => result.current.continueFromDevice());
    expect(deviceForm.setFieldValue).toHaveBeenCalledWith(
      "customerId",
      "customer",
    );
    expect(values.setSubmitting).toHaveBeenNthCalledWith(1, true);
    expect(values.setError).toHaveBeenCalledWith("device save failed");
    expect(values.setSubmitting).toHaveBeenLastCalledWith(false);
  });
});
