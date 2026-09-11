import { invoke as tauriInvoke } from "@tauri-apps/api/core";

export class CommandError extends Error {
  readonly code: string;
  readonly field?: string;

  constructor(code: string, message: string, field?: string) {
    super(message);
    this.name = "CommandError";
    this.code = code;
    if (field) {
      this.field = field;
    }
  }
}

export function isCommandError(error: unknown): error is CommandError {
  return error instanceof CommandError;
}

export function isUnauthorizedError(error: unknown): boolean {
  return isCommandError(error) && error.code === "unauthorized";
}

type UnauthorizedListener = () => void;
const unauthorizedListeners = new Set<UnauthorizedListener>();

export function subscribeUnauthorized(
  listener: UnauthorizedListener,
): () => void {
  unauthorizedListeners.add(listener);
  return () => {
    unauthorizedListeners.delete(listener);
  };
}

function notifyUnauthorized() {
  unauthorizedListeners.forEach((listener) => {
    listener();
  });
}

function asRecord(value: unknown): Record<string, unknown> | null {
  if (value && typeof value === "object") {
    return value as Record<string, unknown>;
  }
  return null;
}

function parseCommandError(error: unknown): CommandError | null {
  if (typeof error === "string") {
    const trimmed = error.trim();
    if (trimmed.startsWith("{")) {
      try {
        return parseCommandError(JSON.parse(trimmed) as unknown);
      } catch {
        return null;
      }
    }
    return null;
  }
  const record = asRecord(error);
  if (!record) {
    return null;
  }
  const code = typeof record.code === "string" ? record.code : null;
  const message = typeof record.message === "string" ? record.message : null;
  if (!code || !message?.trim()) {
    return null;
  }
  const field = typeof record.field === "string" ? record.field : undefined;
  return new CommandError(code, message, field);
}

function errorMessage(error: unknown): string | null {
  if (typeof error === "string" && error.trim()) {
    return error;
  }
  if (error instanceof Error && error.message.trim()) {
    return error.message;
  }
  const record = asRecord(error);
  if (record) {
    if (typeof record.message === "string" && record.message.trim()) {
      return record.message;
    }
    if (typeof record.error === "string" && record.error.trim()) {
      return record.error;
    }
  }
  return null;
}

function isTauriRuntimeAvailable(): boolean {
  if (typeof window === "undefined") {
    return false;
  }
  return Boolean(
    (window as unknown as { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__,
  );
}

export async function invoke<T>(
  command: string,
  args?: Record<string, unknown>,
): Promise<T> {
  if (!isTauriRuntimeAvailable()) {
    throw new Error(
      "Tauri IPC is unavailable. Run the app with `npm run tauri dev` (desktop window), not a browser tab on localhost.",
    );
  }
  try {
    return await tauriInvoke<T>(command, args);
  } catch (error) {
    throw mapInvokeError(error, command);
  }
}

export function mapInvokeError(error: unknown, command: string): Error {
  const commandError = parseCommandError(error);
  if (commandError) {
    if (commandError.code === "unauthorized") notifyUnauthorized();
    return commandError;
  }
  return new Error(errorMessage(error) ?? `Command failed: ${command}`);
}
