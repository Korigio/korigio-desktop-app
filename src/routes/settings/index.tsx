import { replace } from "react-router";

export function clientLoader() {
  return replace("/settings/general");
}

export default function SettingsIndexRoute() {
  return null;
}
