import { invoke } from "@/shared/api/invoke";
import type {
  Customer,
  CustomerInput,
  CustomerListQuery,
  CustomerListResult,
} from "@/features/customers/types/customer";

export const customersApi = {
  list(query: CustomerListQuery = {}): Promise<CustomerListResult> {
    return invoke<CustomerListResult>("list_customers", { query });
  },
  get(id: string): Promise<Customer> {
    return invoke<Customer>("get_customer", { id });
  },
  create(input: CustomerInput): Promise<Customer> {
    return invoke<Customer>("create_customer", { input });
  },
  update(id: string, input: CustomerInput): Promise<Customer> {
    return invoke<Customer>("update_customer", { id, input });
  },
  archive(id: string): Promise<Customer> {
    return invoke<Customer>("archive_customer", { id });
  },
  unarchive(id: string): Promise<Customer> {
    return invoke<Customer>("unarchive_customer", { id });
  },
};
