import { act, renderHook, waitFor } from "@testing-library/react";
import type { ReactNode } from "react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { Session } from "@/features/staff/types/staff";

const mocks = vi.hoisted(() => ({
  getCurrentSession: vi.fn(),
  unauthorizedListener: undefined as (() => void) | undefined,
}));
vi.mock("@/features/staff/api/staffApi", () => ({
  staffApi: { getCurrentSession: mocks.getCurrentSession },
}));
vi.mock("@/shared/api/invoke", () => ({
  subscribeUnauthorized: (listener: () => void) => {
    mocks.unauthorizedListener = listener;
    return () => {
      mocks.unauthorizedListener = undefined;
    };
  },
}));

import { useSession } from "./session-context";
import { SessionProvider } from "./useSession";

const session = {
  staff: { id: "staff", role: "admin", name: "Ada" },
  deviceId: "device",
} as Session;
const wrapper = ({ children }: { children: ReactNode }) => (
  <SessionProvider>{children}</SessionProvider>
);

describe("session and team write gate state", () => {
  beforeEach(() => {
    mocks.getCurrentSession.mockReset();
    mocks.unauthorizedListener = undefined;
  });

  it("loads and applies a signed-in session", async () => {
    mocks.getCurrentSession.mockResolvedValue(null);
    const { result } = renderHook(() => useSession(), { wrapper });
    await waitFor(() => expect(result.current.loading).toBe(false));
    expect(result.current.session).toBeNull();
    act(() => result.current.applySession(session));
    expect(result.current.session).toBe(session);
    expect(result.current.error).toBeNull();
  });

  it("clears stale identity on load failure", async () => {
    mocks.getCurrentSession.mockRejectedValue(new Error("offline"));
    const { result } = renderHook(() => useSession(), { wrapper });
    await waitFor(() => expect(result.current.loading).toBe(false));
    expect(result.current.session).toBeNull();
    expect(result.current.error).toBe("offline");
  });

  it("refreshes the gate after an unauthorized command", async () => {
    mocks.getCurrentSession
      .mockResolvedValueOnce(session)
      .mockResolvedValueOnce(null);
    const { result } = renderHook(() => useSession(), { wrapper });
    await waitFor(() => expect(result.current.session).toBe(session));
    act(() => mocks.unauthorizedListener?.());
    await waitFor(() => expect(result.current.session).toBeNull());
    expect(mocks.getCurrentSession).toHaveBeenCalledTimes(2);
  });
});
