import { act, renderHook, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { Company } from "@/features/companies/types/company";

const mocks = vi.hoisted(() => ({ list: vi.fn() }));
vi.mock("@/features/companies/api/companiesApi", () => ({
  companiesApi: { list: mocks.list },
}));

import { useRepairIntakeCompanies } from "./useRepairIntakeCompanies";

const primary = {
  id: "primary",
  legalName: "Primary",
  isDefault: true,
} as Company;
const secondary = {
  id: "secondary",
  legalName: "Secondary",
  isDefault: false,
} as Company;

describe("repair intake company gate", () => {
  beforeEach(() => mocks.list.mockReset());

  it("opens intake with the preferred active company", async () => {
    mocks.list.mockResolvedValue({ items: [secondary, primary] });
    const onSelect = vi.fn();
    const { result } = renderHook(() => useRepairIntakeCompanies(onSelect));
    expect(result.current.gate).toBe("loading");
    await waitFor(() => expect(result.current.gate).toBe("ready"));
    expect(result.current.company).toBe(primary);
    act(() => result.current.selectCompany(secondary));
    expect(result.current.company).toBe(secondary);
    expect(onSelect).toHaveBeenCalledOnce();
    act(() => result.current.resetCompany());
    expect(result.current.company).toBe(primary);
  });

  it("keeps the gate closed when there are no active companies", async () => {
    mocks.list.mockResolvedValue({ items: [] });
    const { result } = renderHook(() => useRepairIntakeCompanies(vi.fn()));
    await waitFor(() => expect(result.current.gate).toBe("no-company"));
    expect(result.current.companies).toEqual([]);
    expect(result.current.company).toBeNull();
    expect(result.current.companiesError).toBeNull();
  });
});
