import { Outlet, redirect } from "react-router";
import { companiesApi } from "@/features/companies/api/companiesApi";

export async function clientLoader() {
  if (!(await companiesApi.hasActiveCompanies())) {
    return redirect("/setup");
  }
  return null;
}

export default function RequireActiveCompanyRoute() {
  return <Outlet />;
}
