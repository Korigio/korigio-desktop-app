import { invoke as tauriInvoke } from "@tauri-apps/api/core";

type CommandFailure = {
  message?: string;
};

export async function invoke<T>(
  command: string,
  args?: Record<string, unknown>,
): Promise<T> {
  try {
    return await tauriInvoke<T>(command, args);
  } catch (error) {
    const failure = error as CommandFailure;
    if (failure && typeof failure.message === "string") {
      throw new Error(failure.message);
    }
    throw error;
  }
}
