import type { Config } from "@react-router/dev/config";

export default {
  appDirectory: "src",
  // Desktop SPA: no runtime SSR. Root is still pre-rendered at build time for index.html.
  ssr: false,
  routeDiscovery: { mode: "initial" },
} satisfies Config;
