export type Customer = {
  id: number;
  name: string;
  phone: string | null;
  email: string | null;
  address: string | null;
  notes: string | null;
  createdAt: string;
  updatedAt: string;
  archivedAt: string | null;
};

export type CustomerInput = {
  name: string;
  phone?: string | null;
  email?: string | null;
  address?: string | null;
  notes?: string | null;
};

export type CustomerListQuery = {
  query?: string;
  includeArchived?: boolean;
  page?: number;
  pageSize?: number;
};

export type CustomerListResult = {
  items: Customer[];
  total: number;
  page: number;
  pageSize: number;
};
