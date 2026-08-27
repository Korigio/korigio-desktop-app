import { useForm } from "@tanstack/react-form";
import { useNavigate } from "react-router";
import { customersApi } from "@/features/customers/api/customersApi";
import type { Customer, CustomerInput } from "@/features/customers/types/customer";

type Options = {
  customer?: Customer;
  mode: "create" | "edit";
};

function toInput(customer?: Customer): CustomerInput {
  return {
    name: customer?.name ?? "",
    phone: customer?.phone ?? "",
    email: customer?.email ?? "",
    address: customer?.address ?? "",
    notes: customer?.notes ?? "",
  };
}

export function useCustomerForm({ customer, mode }: Options) {
  const navigate = useNavigate();

  const form = useForm({
    defaultValues: toInput(customer),
    onSubmit: async ({ value }) => {
      const input: CustomerInput = {
        name: value.name,
        phone: value.phone || null,
        email: value.email || null,
        address: value.address || null,
        notes: value.notes || null,
      };

      if (mode === "create") {
        const created = await customersApi.create(input);
        navigate(`/customers/${created.id}`);
        return;
      }

      if (!customer) {
        throw new Error("Missing customer for edit");
      }
      const updated = await customersApi.update(customer.id, input);
      navigate(`/customers/${updated.id}`);
    },
  });

  return form;
}
