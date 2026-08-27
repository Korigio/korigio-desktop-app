import { useEffect } from "react";
import { useNavigate } from "react-router";
import { listen } from "@tauri-apps/api/event";

const OPEN_SETTINGS_EVENT = "open-settings";
const OPEN_SETTINGS_DOM_EVENT = "rm:open-settings";
const OPEN_REPAIR_INTAKE_EVENT = "open-repair-intake";
const OPEN_REPAIR_INTAKE_DOM_EVENT = "rm:open-repair-intake";

/** Navigates when native desktop menu items are chosen. */
export function NativeMenuBridge() {
  const navigate = useNavigate();

  useEffect(() => {
    const onSettingsDom = () => {
      navigate("/settings");
    };
    const onIntakeDom = () => {
      navigate("/repairs/intake");
    };

    window.addEventListener(OPEN_SETTINGS_DOM_EVENT, onSettingsDom);
    window.addEventListener(OPEN_REPAIR_INTAKE_DOM_EVENT, onIntakeDom);

    let unlistenSettings: (() => void) | undefined;
    let unlistenIntake: (() => void) | undefined;

    void listen(OPEN_SETTINGS_EVENT, () => navigate("/settings"))
      .then((fn) => {
        unlistenSettings = fn;
      })
      .catch((error: unknown) => {
        console.error("Failed to subscribe to Settings menu event", error);
      });

    void listen(OPEN_REPAIR_INTAKE_EVENT, () => navigate("/repairs/intake"))
      .then((fn) => {
        unlistenIntake = fn;
      })
      .catch((error: unknown) => {
        console.error("Failed to subscribe to repair intake menu event", error);
      });

    return () => {
      window.removeEventListener(OPEN_SETTINGS_DOM_EVENT, onSettingsDom);
      window.removeEventListener(OPEN_REPAIR_INTAKE_DOM_EVENT, onIntakeDom);
      unlistenSettings?.();
      unlistenIntake?.();
    };
  }, [navigate]);

  return null;
}
