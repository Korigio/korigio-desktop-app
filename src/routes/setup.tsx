import { redirect } from "react-router";
import { companiesApi } from "@/features/companies/api/companiesApi";
import { FirstRunSetupPage } from "@/features/setup/pages/FirstRunSetupPage";

export async function clientLoader() {
  if (await companiesApi.hasActiveCompanies()) {
    return redirect("/");
  }
  return null;
}

export default function SetupRoute() {
  return <FirstRunSetupPage />;
}
