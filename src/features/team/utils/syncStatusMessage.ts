/** Exact `errorMessage` from Rust when a TCP connect fails. */
export const SYNC_UNREACHABLE_MESSAGE =
  "Could not reach the other computer on the network.";

export function syncErrorDisplay(
  errorMessage: string,
  unreachableLabel: string,
): string {
  return errorMessage === SYNC_UNREACHABLE_MESSAGE
    ? unreachableLabel
    : errorMessage;
}
