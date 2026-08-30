import { NavLink } from "react-router";
import { settingsCategories } from "@/features/settings/constants/settingsCategories";
import { Card } from "@/ui";
import { useI18n } from "@/shared/hooks/useI18n";
import { cn } from "@/shared/utils/cn";

export function SettingsNav() {
  const { t } = useI18n();

  return (
    <Card className="border-primary/20 bg-surface">
      <nav aria-label={t("settings.nav.label")} className="flex flex-row flex-wrap gap-1">
        {settingsCategories.map((category) => (
          <NavLink
            key={category.path}
            to={category.path}
            end
            className={({ isActive }) =>
              cn(
                "rounded-md px-3 py-2 text-sm",
                isActive
                  ? "bg-background font-medium text-foreground"
                  : "text-foreground hover:bg-background",
              )
            }
          >
            {t(category.labelKey)}
          </NavLink>
        ))}
      </nav>
    </Card>
  );
}
