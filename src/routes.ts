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
  ]),
] satisfies RouteConfig;
