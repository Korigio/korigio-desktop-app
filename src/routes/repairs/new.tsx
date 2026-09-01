import { replace } from "react-router";
import type { Route } from "./+types/new";

export function clientLoader({ request }: Route.ClientLoaderArgs) {
  const search = new URL(request.url).search;
  return replace(search ? `/repairs/intake${search}` : "/repairs/intake");
}

export default function RepairNewRedirectRoute() {
  return null;
}
