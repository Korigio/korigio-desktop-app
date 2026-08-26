import {
  type RouteConfig,
  index,
} from "@react-router/dev/routes";

/**
 * Framework-mode route config (React Router 7).
 * Prefer `index`, `route`, `layout`, and `prefix` from `@react-router/dev/routes`.
 * Route modules under `src/routes/` stay thin and render feature pages.
 */
export default [
  index("./routes/home.tsx"),
] satisfies RouteConfig;
