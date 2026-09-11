import {
  isRouteErrorResponse,
  Links,
  Meta,
  Outlet,
  Scripts,
  ScrollRestoration,
} from "react-router";
import type { Route } from "./+types/root";
import { I18nProvider } from "@/app/providers/I18nProvider";
import { ThemeProvider } from "@/app/providers/ThemeProvider";
import "@/styles/globals.css";

export function Layout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="en">
      <head>
        <meta charSet="utf-8" />
        <meta name="viewport" content="width=device-width, initial-scale=1" />
        <Meta />
        <Links />
      </head>
      <body>
        <ThemeProvider>
          <I18nProvider>{children}</I18nProvider>
        </ThemeProvider>
        <ScrollRestoration />
        <Scripts />
      </body>
    </html>
  );
}

export function HydrateFallback() {
  return (
    <main className="flex min-h-full flex-col items-center justify-center gap-2 bg-background p-8 text-foreground">
      <p className="text-lg font-semibold">Korigio</p>
      <p className="text-sm text-muted">Loading…</p>
    </main>
  );
}

export default function App() {
  return <Outlet />;
}

export function ErrorBoundary({ error }: Route.ErrorBoundaryProps) {
  let message = "Error";
  let details = "An unexpected error occurred.";
  let stack: string | undefined;

  if (isRouteErrorResponse(error)) {
    message = error.status === 404 ? "404" : "Error";
    details =
      error.status === 404
        ? "The requested page could not be found."
        : error.statusText || details;
  } else if (import.meta.env.DEV && error instanceof Error) {
    details = error.message;
    stack = error.stack;
  }

  return (
    <main className="flex min-h-full flex-col items-center justify-center gap-2 bg-background p-8 text-foreground">
      <h1 className="text-2xl font-semibold">{message}</h1>
      <p className="max-w-xl text-center text-muted">{details}</p>
      {import.meta.env.DEV ? (
        <p className="mt-2 max-w-xl text-center text-sm text-muted">
          Development UI must run inside the desktop window via{" "}
          <code className="rounded bg-surface px-1 py-0.5 text-foreground">
            npm run tauri dev
          </code>
          , not a normal browser tab on localhost.
        </p>
      ) : null}
      {stack ? (
        <pre className="mt-4 max-w-full overflow-x-auto rounded-md border border-border bg-surface p-4 text-xs">
          <code>{stack}</code>
        </pre>
      ) : null}
    </main>
  );
}
