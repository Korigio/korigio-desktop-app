import {
  type RouteConfig,
  index,
  layout,
  route,
} from "@react-router/dev/routes";

/**
 * Framework-mode route config (React Router 7).
 * Prefer `index`, `route`, `layout`, and `prefix` from `@react-router/dev/routes`.
 */
export default [
  layout("./routes/app-shell.tsx", [
    index("./routes/home.tsx"),
    route("customers", "./routes/customers/list.tsx"),
    route("customers/new", "./routes/customers/new.tsx"),
    route("customers/:id", "./routes/customers/detail.tsx"),
    route("customers/:id/edit", "./routes/customers/edit.tsx"),
    route("devices", "./routes/devices/list.tsx"),
    route("devices/new", "./routes/devices/new.tsx"),
    route("devices/:id", "./routes/devices/detail.tsx"),
    route("devices/:id/edit", "./routes/devices/edit.tsx"),
  ]),
] satisfies RouteConfig;
