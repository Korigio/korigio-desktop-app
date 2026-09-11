import { act, renderHook } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import type { Repair } from "@/features/repairs/types/repair";
import {
  modalKeyFromAction,
  useRepairDetailWorkflowModals,
} from "./useRepairDetailWorkflowModals";

describe("repair detail modal routing", () => {
  it.each([
    ["intake", "confirmIntake"],
    ["startDiagnose", "diagnose"],
    ["adjustDiagnosis", "diagnose"],
    ["approval", "confirmCustomerApproval"],
    ["parts", "confirmParts"],
    ["protocol", "writeProtocol"],
    ["summary", "confirmSummary"],
    ["pickup", "recordPickup"],
    ["unknown", null],
    [null, null],
  ])("maps action %s to modal %s", (action, modal) => {
    expect(modalKeyFromAction(action)).toBe(modal);
  });

  it("opens URL action once and removes only the action query", () => {
    const setSearchParams = vi.fn();
    const setRepair = vi.fn();
    const params = new URLSearchParams("action=diagnose&from=list");
    const { result } = renderHook(() =>
      useRepairDetailWorkflowModals(params, setSearchParams, setRepair),
    );
    expect(result.current.openModal).toBe("diagnose");
    expect(setSearchParams).toHaveBeenCalledOnce();
    const [next, options] = setSearchParams.mock.calls[0] as [
      URLSearchParams,
      object,
    ];
    expect(next.toString()).toBe("from=list");
    expect(options).toEqual({ replace: true });
  });

  it("chains workflow modals only for matching next statuses", () => {
    const setRepair = vi.fn();
    const { result } = renderHook(() =>
      useRepairDetailWorkflowModals(new URLSearchParams(), vi.fn(), setRepair),
    );
    act(() =>
      result.current.diagnosisFinalized({
        status: "waiting_customer",
      } as Repair),
    );
    expect(result.current.openModal).toBe("confirmCustomerApproval");
    act(() => result.current.protocolCompleted({ status: "ready" } as Repair));
    expect(result.current.openModal).toBe("confirmSummary");
    act(() =>
      result.current.summaryConfirmed({ status: "awaiting_pickup" } as Repair),
    );
    expect(result.current.openModal).toBe("recordPickup");
    expect(setRepair).toHaveBeenCalledTimes(3);
  });
});
