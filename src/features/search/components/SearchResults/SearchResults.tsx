import { Link } from "react-router";
import { SearchResultSection } from "@/features/search/components/SearchResultSection";
import type { GlobalSearchResult } from "@/features/search/types/search";
import { deviceLabel } from "@/features/devices/types/device";
import { StatusMessage } from "@/ui";
import { useI18n } from "@/shared/hooks/useI18n";

export type SearchResultsProps = {
  result: GlobalSearchResult;
  loading: boolean;
  error: string | null;
  hasQuery: boolean;
};

export function SearchResults({
  result,
  loading,
  error,
  hasQuery,
}: SearchResultsProps) {
  const { t } = useI18n();

  if (error) {
    return <StatusMessage tone="danger">{error}</StatusMessage>;
  }

  if (!hasQuery) {
    return <StatusMessage>{t("search.prompt")}</StatusMessage>;
  }

  if (loading) {
    return <StatusMessage>{t("common.loading")}</StatusMessage>;
  }

  const total =
    result.customers.length + result.devices.length + result.repairs.length;

  if (total === 0) {
    return <StatusMessage>{t("search.empty")}</StatusMessage>;
  }

  return (
    <div className="flex flex-col gap-6">
      <SearchResultSection
        title={t("search.sections.customers")}
        emptyMessage={t("search.sectionEmpty")}
        isEmpty={result.customers.length === 0}
      >
        {result.customers.map((customer) => (
          <li key={customer.id}>
            <Link
              className="block px-3 py-2 font-medium text-primary hover:bg-background hover:underline"
              to={`/customers/${customer.id}`}
            >
              {customer.name}
            </Link>
          </li>
        ))}
      </SearchResultSection>

      <SearchResultSection
        title={t("search.sections.devices")}
        emptyMessage={t("search.sectionEmpty")}
        isEmpty={result.devices.length === 0}
      >
        {result.devices.map((device) => (
          <li key={device.id}>
            <Link
              className="block px-3 py-2 font-medium text-primary hover:bg-background hover:underline"
              to={`/devices/${device.id}`}
            >
              {deviceLabel(device)}
            </Link>
          </li>
        ))}
      </SearchResultSection>

      <SearchResultSection
        title={t("search.sections.repairs")}
        emptyMessage={t("search.sectionEmpty")}
        isEmpty={result.repairs.length === 0}
      >
        {result.repairs.map((repair) => (
          <li key={repair.id}>
            <Link
              className="block px-3 py-2 font-medium text-primary hover:bg-background hover:underline"
              to={`/repairs/${repair.id}`}
            >
              {repair.repairNumber}
            </Link>
          </li>
        ))}
      </SearchResultSection>
    </div>
  );
}
