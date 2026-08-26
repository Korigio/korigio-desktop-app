import { NavLink, Outlet } from "react-router";
import { useI18n } from "@/shared/hooks/useI18n";
import { cn } from "@/shared/utils/cn";

export function AppShell() {
  const { t } = useI18n();

  return (
    <div className="flex min-h-full bg-background text-foreground">
      <aside className="flex w-56 shrink-0 flex-col border-r border-border bg-surface p-4">
        <p className="text-lg font-semibold">{t("app.name")}</p>
        <nav className="mt-6 flex flex-col gap-1">
          <NavLink
            to="/"
            end
            className={({ isActive }) =>
              cn(
                "rounded-md px-3 py-2 text-sm",
                isActive
                  ? "bg-primary text-primary-foreground"
                  : "text-foreground hover:bg-background",
              )
            }
          >
            {t("nav.home")}
          </NavLink>
          <NavLink
            to="/customers"
            className={({ isActive }) =>
              cn(
                "rounded-md px-3 py-2 text-sm",
                isActive
                  ? "bg-primary text-primary-foreground"
                  : "text-foreground hover:bg-background",
              )
            }
          >
            {t("nav.customers")}
          </NavLink>
        </nav>
      </aside>
      <div className="min-w-0 flex-1">
        <Outlet />
      </div>
    </div>
  );
}
