import {
  useCallback,
  useEffect,
  useLayoutEffect,
  useMemo,
  useState,
  type ReactNode,
} from "react";
import {
  ThemeContext,
  type ResolvedTheme,
} from "@/app/providers/theme-context";
import {
  settingsApi,
  type ThemePreference,
} from "@/features/settings/api/settingsApi";

function toPreference(value: string): ThemePreference {
  if (value === "system" || value === "light" || value === "dark") {
    return value;
  }
  return "system";
}

function systemTheme(): ResolvedTheme {
  return window.matchMedia("(prefers-color-scheme: dark)").matches
    ? "dark"
    : "light";
}

function resolveTheme(preference: ThemePreference): ResolvedTheme {
  return preference === "system" ? systemTheme() : preference;
}

function applyHtmlTheme(theme: ResolvedTheme) {
  document.documentElement.classList.toggle("dark", theme === "dark");
}

export function ThemeProvider({ children }: { children: ReactNode }) {
  const [preference, setPreferenceState] = useState<ThemePreference>("system");
  const [resolvedTheme, setResolvedTheme] = useState<ResolvedTheme>(() =>
    typeof window === "undefined" ? "light" : systemTheme(),
  );

  const applyPreference = useCallback((next: ThemePreference) => {
    const resolved = resolveTheme(next);
    applyHtmlTheme(resolved);
    setResolvedTheme(resolved);
  }, []);

  useLayoutEffect(() => {
    applyPreference("system");
  }, [applyPreference]);

  useEffect(() => {
    let cancelled = false;
    void settingsApi
      .getThemeSettings()
      .then((settings) => {
        if (cancelled) {
          return;
        }
        const next = toPreference(settings.preference);
        setPreferenceState(next);
        applyPreference(next);
      })
      .catch(() => {
        if (cancelled) {
          return;
        }
        setPreferenceState("system");
        applyPreference("system");
      });
    return () => {
      cancelled = true;
    };
  }, [applyPreference]);

  useEffect(() => {
    if (preference !== "system") {
      return;
    }
    const media = window.matchMedia("(prefers-color-scheme: dark)");
    const onChange = () => {
      applyPreference("system");
    };
    media.addEventListener("change", onChange);
    return () => {
      media.removeEventListener("change", onChange);
    };
  }, [preference, applyPreference]);

  const setPreference = useCallback(
    async (next: ThemePreference) => {
      const settings = await settingsApi.setThemePreference(next);
      const saved = toPreference(settings.preference);
      setPreferenceState(saved);
      applyPreference(saved);
    },
    [applyPreference],
  );

  const value = useMemo(
    () => ({ preference, resolvedTheme, setPreference }),
    [preference, resolvedTheme, setPreference],
  );

  return (
    <ThemeContext.Provider value={value}>{children}</ThemeContext.Provider>
  );
}
