import { Navigate, useSearchParams } from "react-router";

export default function RepairNewRedirectRoute() {
  const [params] = useSearchParams();
  const search = params.toString();
  return (
    <Navigate
      to={search ? `/repairs/intake?${search}` : "/repairs/intake"}
      replace
    />
  );
}
