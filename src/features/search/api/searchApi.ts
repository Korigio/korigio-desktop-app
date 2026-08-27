import { invoke } from "@/shared/api/invoke";
import type {
  GlobalSearchQuery,
  GlobalSearchResult,
} from "@/features/search/types/search";

export const searchApi = {
  globalSearch(query: GlobalSearchQuery): Promise<GlobalSearchResult> {
    return invoke<GlobalSearchResult>("global_search", { query });
  },
};
