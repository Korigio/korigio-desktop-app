import { useState } from "react";
import { NavLink, Outlet } from "react-router";
import type { LucideIcon } from "lucide-react";
import {
  Building2,
  ClipboardList,
  Home,
  MonitorSmartphone,
  PanelLeft,
  PanelLeftClose,
  Settings,
  Users,
  UsersRound,
  Wrench,
} from "lucide-react";
import { NativeMenuBridge } from "@/app/NativeMenuBridge";
import { AutoBackupOnStartup } from "@/features/settings/components/AutoBackupOnStartup";
import { AppUpdateProvider } from "@/features/settings/hooks/useAppUpdate";
import { useAppUpdate } from "@/features/settings/hooks/app-update-context";
import { SessionProvider } from "@/features/staff/hooks/useSession";
import { Button } from "@/ui/atoms/Button";
import { LinkButton } from "@/ui/molecules/LinkButton";
import { useI18n } from "@/shared/hooks/useI18n";
import { cn } from "@/shared/utils/cn";

const SIDEBAR_COLLAPSED_KEY = "servioo.sidebarCollapsed";

type NavItemConfig = {
  to: string;
  end?: boolean;
  labelKey: string;
  icon: LucideIcon;
  showUpdateBadge?: boolean;
};

const PRIMARY_NAV_ITEMS: NavItemConfig[] = [
  { to: "/", end: true, labelKey: "nav.home", icon: Home },
  { to: "/customers", labelKey: "nav.customers", icon: Users },
  { to: "/devices", labelKey: "nav.devices", icon: MonitorSmartphone },
  { to: "/repairs", labelKey: "nav.repairs", icon: Wrench },
  {
    to: "/diagnosis-templates",
    labelKey: "nav.diagnosisTemplates",
    icon: ClipboardList,
  },
];

const SECONDARY_NAV_ITEMS: NavItemConfig[] = [
  { to: "/companies", labelKey: "nav.companies", icon: Building2 },
  { to: "/team", labelKey: "nav.team", icon: UsersRound },
  {
    to: "/settings",
    labelKey: "nav.settings",
    icon: Settings,
    showUpdateBadge: true,
  },
];

function readSidebarCollapsed(): boolean {
  try {
    return localStorage.getItem(SIDEBAR_COLLAPSED_KEY) === "true";
  } catch {
    return false;
  }
}

function persistSidebarCollapsed(collapsed: boolean) {
  try {
    localStorage.setItem(SIDEBAR_COLLAPSED_KEY, String(collapsed));
  } catch {
    // Ignore quota / private-mode failures.
  }
}

type NavItemProps = {
  to: string;
  end?: boolean;
  label: string;
  icon: LucideIcon;
  collapsed: boolean;
  updateAvailable?: boolean;
  updateBadgeLabel?: string;
};

function NavItem({
  to,
  end,
  label,
  icon: Icon,
  collapsed,
  updateAvailable,
  updateBadgeLabel,
}: NavItemProps) {
  return (
    <NavLink
      to={to}
      end={end}
      prefetch="render"
      title={collapsed ? label : undefined}
      aria-label={collapsed ? label : undefined}
      className={({ isActive }) =>
        cn(
          "inline-flex items-center gap-2 whitespace-nowrap rounded-lg px-3 py-2 text-sm",
          collapsed && "md:justify-center md:px-2",
          isActive
            ? "bg-background font-medium text-foreground"
            : "text-foreground hover:bg-background/80",
        )
      }
    >
      <Icon className="size-4 shrink-0" aria-hidden />
      <span className={cn(collapsed && "md:hidden")}>{label}</span>
      {updateAvailable ? (
        <span
          className="size-2 shrink-0 rounded-full bg-primary"
          aria-label={updateBadgeLabel}
        />
      ) : null}
    </NavLink>
  );
}

export function AppShell() {
  return (
    <SessionProvider>
      <AppUpdateProvider>
        <AppShellLayout />
      </AppUpdateProvider>
    </SessionProvider>
  );
}

function AppShellLayout() {
  const { t } = useI18n();
  const { updateAvailable } = useAppUpdate();
  const [collapsed, setCollapsed] = useState(readSidebarCollapsed);

  function toggleCollapsed() {
    setCollapsed((prev) => {
      const next = !prev;
      persistSidebarCollapsed(next);
      return next;
    });
  }

  const toggleLabel = collapsed ? t("nav.expand") : t("nav.collapse");
  const ToggleIcon = collapsed ? PanelLeft : PanelLeftClose;

  return (
    <div className="flex h-full flex-col overflow-hidden bg-background text-foreground md:flex-row">
      <NativeMenuBridge />
      <AutoBackupOnStartup />
      <aside
        className={cn(
          "shrink-0 border-b border-border bg-sidebar p-3 transition-[width] duration-200 ease-out md:min-h-0 md:overflow-y-auto md:border-b-0 md:border-r md:p-3",
          collapsed ? "md:w-14" : "md:w-64 md:p-4",
        )}
      >
        <div
          className={cn(
            "flex items-center gap-2",
            collapsed ? "md:justify-center" : "justify-between",
          )}
        >
          <p className={cn("text-lg font-semibold", collapsed && "md:hidden")}>
            {t("app.name")}
          </p>
          <Button
            type="button"
            variant="secondary"
            className="hidden size-8 shrink-0 p-0 md:inline-flex"
            aria-label={toggleLabel}
            title={toggleLabel}
            onClick={toggleCollapsed}
          >
            <ToggleIcon className="size-4" aria-hidden />
          </Button>
        </div>
        <nav
          className={cn(
            "mt-3 flex gap-1 overflow-x-auto pb-1 md:mt-6 md:flex-col md:overflow-visible md:pb-0",
            collapsed && "md:items-stretch",
          )}
        >
          {PRIMARY_NAV_ITEMS.map((item) => (
            <NavItem
              key={item.to}
              to={item.to}
              end={item.end}
              label={t(item.labelKey)}
              icon={item.icon}
              collapsed={collapsed}
            />
          ))}
          <div
            role="separator"
            className="w-px shrink-0 self-stretch bg-border md:h-px md:w-full"
          />
          {SECONDARY_NAV_ITEMS.map((item) => (
            <NavItem
              key={item.to}
              to={item.to}
              end={item.end}
              label={t(item.labelKey)}
              icon={item.icon}
              collapsed={collapsed}
              updateAvailable={
                item.showUpdateBadge ? updateAvailable : undefined
              }
              updateBadgeLabel={
                item.showUpdateBadge
                  ? t("nav.settingsUpdateAvailable")
                  : undefined
              }
            />
          ))}
        </nav>
      </aside>

      <div className="flex min-h-0 min-w-0 flex-1 flex-col overflow-hidden">
        <header className="shrink-0 border-b border-border bg-surface px-3 py-2 sm:px-4">
          <div className="flex flex-wrap items-center justify-end gap-2">
            <LinkButton
              to="/repairs/intake"
              prefetch="render"
              className="px-3 py-1.5 text-xs sm:text-sm"
            >
              {t("repairs.intake.start")}
            </LinkButton>
          </div>
        </header>
        <div className="min-h-0 min-w-0 flex-1 overflow-y-auto overscroll-contain">
          <Outlet />
        </div>
      </div>
    </div>
  );
}
