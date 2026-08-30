import { createContext } from "react";
import type { ThemePreference } from "@/features/settings/api/settingsApi";

export type ResolvedTheme = "light" | "dark";

export type ThemeContextValue = {
  preference: ThemePreference;
  resolvedTheme: ResolvedTheme;
  setPreference: (preference: ThemePreference) => Promise<void>;
};

export const ThemeContext = createContext<ThemeContextValue | null>(null);
