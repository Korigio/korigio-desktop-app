export type PrintDiagnosisValue = boolean | string;

export type PrintDiagnosisItem = {
  id: string;
  label: string;
  kind: string;
  value: PrintDiagnosisValue;
};

export type PrintCompany = {
  id: number;
  legalName: string;
  tradeName: string | null;
  taxId: string | null;
  address: string | null;
  phone: string | null;
  email: string | null;
  website: string | null;
};

export type PrintCustomer = {
  id: number;
  name: string;
  phone: string | null;
  email: string | null;
  address: string | null;
};

export type PrintDevice = {
  id: number;
  deviceType: string;
  manufacturer: string | null;
  model: string | null;
  serialNumber: string | null;
};

export type RepairPrintReport = {
  repair: {
    id: number;
    repairNumber: string;
    status: string;
    receivedAt: string;
    reportedProblem: string | null;
    accessoriesReceived: string | null;
    deviceCondition: string | null;
    expectedPickupAt: string | null;
    diagnosisNotes: string | null;
    workPerformed: string | null;
    notes: string | null;
    readyAt: string | null;
    collectedAt: string | null;
  };
  customer: PrintCustomer;
  device: PrintDevice;
  company: PrintCompany | null;
  companyLogoAbsolutePath: string | null;
  diagnosis: {
    items: PrintDiagnosisItem[];
  } | null;
};

export type SummaryPrintReport = {
  repair: {
    id: number;
    repairNumber: string;
    status: string;
    receivedAt: string;
    reportedProblem: string | null;
    accessoriesReceived: string | null;
    deviceCondition: string | null;
    workPerformed: string | null;
    expectedPickupAt: string | null;
    readyAt: string | null;
    collectedAt: string | null;
    estimateBaseCents: number | null;
    estimateTaxRateBps: number | null;
    estimateTaxCents: number | null;
    estimateGrossCents: number | null;
  };
  customer: PrintCustomer;
  device: PrintDevice;
  company: PrintCompany | null;
  companyLogoAbsolutePath: string | null;
  currency: string;
};

export type DiagnosisPrintReport = {
  repair: {
    id: number;
    repairNumber: string;
    status: string;
    diagnosisNotes: string | null;
    expectedPickupAt: string | null;
    estimateBaseCents: number | null;
    estimateTaxRateBps: number | null;
    estimateTaxCents: number | null;
    estimateGrossCents: number | null;
  };
  customer: PrintCustomer;
  device: PrintDevice;
  company: PrintCompany | null;
  companyLogoAbsolutePath: string | null;
  currency: string;
};
