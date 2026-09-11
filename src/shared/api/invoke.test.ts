import { beforeEach, describe, expect, it, vi } from "vitest";

const { tauriInvoke } = vi.hoisted(() => ({ tauriInvoke: vi.fn() }));
vi.mock("@tauri-apps/api/core", () => ({ invoke: tauriInvoke }));

import {
  CommandError,
  invoke,
  isCommandError,
  isUnauthorizedError,
  mapInvokeError,
  subscribeUnauthorized,
} from "./invoke";

describe("shared invoke error mapping", () => {
  beforeEach(() => {
    tauriInvoke.mockReset();
    (
      window as unknown as { __TAURI_INTERNALS__?: { invoke: unknown } }
    ).__TAURI_INTERNALS__ = { invoke: vi.fn() };
  });

  it("returns successful command results", async () => {
    tauriInvoke.mockResolvedValue({ ok: true });
    await expect(invoke("status", { id: "1" })).resolves.toEqual({ ok: true });
    expect(tauriInvoke).toHaveBeenCalledWith("status", { id: "1" });
  });

  it("rejects clearly when the Tauri runtime is missing", async () => {
    delete (window as unknown as { __TAURI_INTERNALS__?: unknown })
      .__TAURI_INTERNALS__;
    await expect(invoke("status")).rejects.toThrow(/npm run tauri dev/);
    expect(tauriInvoke).not.toHaveBeenCalled();
  });

  it.each([
    [{ code: "validation", message: "Bad value", field: "name" }],
    ['{"code":"validation","message":"Bad value","field":"name"}'],
  ])(
    "maps structured errors from object and JSON payloads",
    async (payload) => {
      const error = mapInvokeError(payload, "save");
      expect(error).toBeInstanceOf(CommandError);
      expect(isCommandError(error)).toBe(true);
      expect(error).toMatchObject({
        code: "validation",
        message: "Bad value",
        field: "name",
      });
    },
  );

  it("notifies active listeners only for unauthorized command errors", async () => {
    const listener = vi.fn();
    const unsubscribe = subscribeUnauthorized(listener);
    mapInvokeError({ code: "validation", message: "No" }, "save");
    expect(listener).not.toHaveBeenCalled();
    const error = mapInvokeError(
      { code: "unauthorized", message: "Sign in" },
      "save",
    );
    expect(isUnauthorizedError(error)).toBe(true);
    expect(listener).toHaveBeenCalledOnce();
    unsubscribe();
    mapInvokeError({ code: "unauthorized", message: "Sign in" }, "save");
    expect(listener).toHaveBeenCalledOnce();
  });

  it("preserves useful plain errors and supplies a command fallback", async () => {
    expect(mapInvokeError({ error: "offline" }, "sync").message).toBe(
      "offline",
    );
    expect(mapInvokeError(null, "sync").message).toBe("Command failed: sync");
  });
});
