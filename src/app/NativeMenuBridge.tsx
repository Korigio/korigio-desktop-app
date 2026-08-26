import { useEffect } from "react";
import { useNavigate } from "react-router";
import { listen } from "@tauri-apps/api/event";

const OPEN_SETTINGS_EVENT = "open-settings";
const OPEN_SETTINGS_DOM_EVENT = "rm:open-settings";

/** Navigates to Settings when the native desktop menu item is chosen. */
export function NativeMenuBridge() {
  const navigate = useNavigate();

  useEffect(() => {
    const goSettings = () => {
      navigate("/settings");
    };

    const onDomEvent = () => {
      goSettings();
    };
    window.addEventListener(OPEN_SETTINGS_DOM_EVENT, onDomEvent);

    let unlisten: (() => void) | undefined;
    void listen(OPEN_SETTINGS_EVENT, goSettings)
      .then((fn) => {
        unlisten = fn;
      })
      .catch((error: unknown) => {
        console.error("Failed to subscribe to Settings menu event", error);
      });

    return () => {
      window.removeEventListener(OPEN_SETTINGS_DOM_EVENT, onDomEvent);
      unlisten?.();
    };
  }, [navigate]);

  return null;
}
