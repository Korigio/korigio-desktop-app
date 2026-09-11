import { renderHook, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { Customer } from "@/features/customers/types/customer";
import type { Device } from "@/features/devices/types/device";

const mocks = vi.hoisted(() => ({ getCustomer: vi.fn(), getDevice: vi.fn() }));
vi.mock("@/features/customers/api/customersApi", () => ({
  customersApi: { get: mocks.getCustomer },
}));
vi.mock("@/features/devices/api/devicesApi", () => ({
  devicesApi: { get: mocks.getDevice },
}));

import { useRepairIntakePresets } from "./useRepairIntakePresets";

const customer = { id: "customer" } as Customer;
const device = { id: "device", customerId: customer.id } as Device;

function params(overrides: Record<string, unknown> = {}) {
  return {
    presetCustomerId: customer.id,
    presetDeviceId: device.id,
    gate: "ready" as const,
    includeCompanyStep: false,
    goTo: vi.fn(),
    selectCustomer: vi.fn(),
    selectDevice: vi.fn(),
    setCustomer: vi.fn(),
    setCreatedCustomerId: vi.fn(),
    setDevice: vi.fn(),
    ...overrides,
  };
}

describe("repair intake preset hydration", () => {
  beforeEach(() => {
    mocks.getCustomer.mockReset();
    mocks.getDevice.mockReset();
    mocks.getCustomer.mockResolvedValue(customer);
    mocks.getDevice.mockResolvedValue(device);
  });

  it("waits for the company gate, then selects matching entities and skips", async () => {
    const values = params({ gate: "loading" });
    const { result, rerender } = renderHook(
      ({ gate }) => useRepairIntakePresets({ ...values, gate }),
      { initialProps: { gate: "loading" as "loading" | "ready" } },
    );
    expect(result.current.presetsReady).toBe(false);
    expect(mocks.getCustomer).not.toHaveBeenCalled();
    rerender({ gate: "ready" });
    await waitFor(() => expect(result.current.presetsReady).toBe(true));
    expect(values.selectCustomer).toHaveBeenCalledWith(customer);
    expect(values.selectDevice).toHaveBeenCalledWith(device);
    expect(values.goTo).toHaveBeenCalledWith("details");
  });

  it("rejects a device belonging to another customer", async () => {
    mocks.getDevice.mockResolvedValue({ ...device, customerId: "other" });
    const values = params();
    const { result } = renderHook(() => useRepairIntakePresets(values));
    await waitFor(() => expect(result.current.presetsReady).toBe(true));
    expect(values.selectDevice).not.toHaveBeenCalled();
    expect(values.goTo).toHaveBeenCalledWith("device");
  });

  it("keeps company selection first when multiple companies exist", async () => {
    const values = params({ includeCompanyStep: true });
    const { result } = renderHook(() => useRepairIntakePresets(values));
    await waitFor(() => expect(result.current.presetsReady).toBe(true));
    expect(values.goTo).toHaveBeenCalledWith("company");
  });
});
