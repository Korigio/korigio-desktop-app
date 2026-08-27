# Functional reference — CP Reparaciones

CP Reparaciones is a **workflow reference only**. We do not copy its code, UI, assets, or Access/MDB architecture.

Reference sites (public descriptions):

- https://www.cpimario.com/index5.html
- https://www.cpimario.com/repair_preview.html

## Problems we solve

| User problem | Our approach |
| --- | --- |
| Track devices in a small workshop | Repair orders as the core entity |
| Returning customers | Customer CRUD, search, archive |
| Known devices / serials | Devices linked to customers; serial search |
| Know work status | Simple status workflow |
| Find records quickly | SQL-side search + indexes |
| Paper for the customer | A4 intake/report (later phase) |
| Don’t lose data | AppData + backup/restore |
| Confidential data | Validation, CSP, restricted capabilities; encryption decision Phase 12 |

## In scope for the phased roadmap

Customers, devices, repairs, diagnosis templates, images, search, backup/restore, printing (first report), security hardening, performance hardening, Windows offline installer, CI release builds.

## Out of scope for v1 (propose only)

Appointments/calendar, multi-user password roles, network/Dropbox sync, inventory, invoices/payments, SMS/email, cloud, automatic updates, customer portal.

## Legacy we will not reproduce

Access `.mdb`, data inside the install folder, “run as administrator” as normal mode, Jet/DAO repair rituals, proprietary icons/UI.
