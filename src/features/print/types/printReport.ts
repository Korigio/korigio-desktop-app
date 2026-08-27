export type PrintDiagnosisValue = boolean | string;

export type PrintDiagnosisItem = {
  id: string;
  label: string;
  kind: string;
  value: PrintDiagnosisValue;
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
    diagnosisNotes: string | null;
    workPerformed: string | null;
    notes: string | null;
    readyAt: string | null;
    collectedAt: string | null;
  };
  customer: {
    id: number;
    name: string;
    phone: string | null;
    email: string | null;
    address: string | null;
  };
  device: {
    id: number;
    deviceType: string;
    manufacturer: string | null;
    model: string | null;
    serialNumber: string | null;
  };
  diagnosis: {
    items: PrintDiagnosisItem[];
  } | null;
};
