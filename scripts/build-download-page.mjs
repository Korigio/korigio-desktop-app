#!/usr/bin/env node
/**
 * Bake the public download page from GitHub Releases.
 *
 *   node scripts/build-download-page.mjs --output website/index.html
 *   node scripts/build-download-page.mjs --input releases.json --output website/index.html
 *   node scripts/build-download-page.mjs --repo Korigio/korigio-downloads --output index.html
 */
import { execFileSync } from "node:child_process";
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname } from "node:path";

const DEFAULT_REPO = "Korigio/korigio-downloads";
const PRODUCT = "Korigio";

function parseArgs(argv) {
  const parsed = { input: undefined, output: undefined, repo: DEFAULT_REPO };
  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    const next = argv[index + 1];
    if (arg === "--input" && next) {
      parsed.input = next;
      index += 1;
    } else if (arg === "--output" && next) {
      parsed.output = next;
      index += 1;
    } else if (arg === "--repo" && next) {
      parsed.repo = next;
      index += 1;
    } else {
      throw new Error(
        "Usage: node scripts/build-download-page.mjs [--input releases.json] [--repo owner/name] --output index.html",
      );
    }
  }
  return parsed;
}

function escapeHtml(value) {
  return String(value)
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;");
}

function formatDate(iso) {
  if (!iso) {
    return "";
  }
  const date = new Date(iso);
  if (Number.isNaN(date.getTime())) {
    return "";
  }
  return new Intl.DateTimeFormat("en-GB", {
    day: "numeric",
    month: "short",
    year: "numeric",
    timeZone: "UTC",
  }).format(date);
}

function versionOf(release) {
  return String(release.tag_name || "").replace(/^v/, "");
}

function pickAsset(assets, test) {
  return (assets ?? []).find((asset) => test(String(asset.name || "").toLowerCase()));
}

function classifyAssets(assets) {
  return {
    windows: pickAsset(
      assets,
      (name) => name.endsWith(".exe") || name.includes("setup"),
    ),
    macos: pickAsset(assets, (name) => name.endsWith(".dmg")),
    linux: pickAsset(
      assets,
      (name) => name.endsWith(".appimage"),
    ),
  };
}

function isListedRelease(release) {
  return Boolean(release) && !release.draft && !release.prerelease;
}

function selectReleases(releases) {
  const listed = (Array.isArray(releases) ? releases : [])
    .filter(isListedRelease)
    .slice()
    .sort((left, right) => {
      const leftTime = Date.parse(left.published_at || left.created_at || 0);
      const rightTime = Date.parse(right.published_at || right.created_at || 0);
      return rightTime - leftTime;
    });
  return {
    latest: listed[0] ?? null,
    previous: listed.slice(1),
  };
}

function loadReleases(input, repo) {
  if (input) {
    return JSON.parse(readFileSync(input, "utf8"));
  }
  const json = execFileSync("gh", ["api", `repos/${repo}/releases`, "--paginate"], {
    encoding: "utf8",
  });
  return JSON.parse(json);
}

function downloadHref(asset) {
  const href = String(asset?.browser_download_url || "");
  if (!href.startsWith("https://github.com/")) {
    return "";
  }
  return href;
}

function card(os, asset, fallbackLabel) {
  const href = downloadHref(asset);
  const disabled = href ? "" : " disabled";
  const link = href || "#";
  const file = asset?.name ? escapeHtml(asset.name) : fallbackLabel;
  return `<a class="card${disabled}" href="${escapeHtml(link)}">
          <span class="os">${escapeHtml(os)}</span>
          <span class="file">${file}</span>
          <span class="btn">Download</span>
        </a>`;
}

function archiveLinks(assets) {
  const parts = [];
  if (assets.windows && downloadHref(assets.windows)) {
    parts.push(
      `<a href="${escapeHtml(downloadHref(assets.windows))}">Windows</a>`,
    );
  }
  if (assets.macos && downloadHref(assets.macos)) {
    parts.push(`<a href="${escapeHtml(downloadHref(assets.macos))}">macOS</a>`);
  }
  if (assets.linux && downloadHref(assets.linux)) {
    parts.push(`<a href="${escapeHtml(downloadHref(assets.linux))}">Linux</a>`);
  }
  return parts.join(" · ") || `<span class="muted">No installers</span>`;
}

function renderArchive(previous) {
  if (previous.length === 0) {
    return "";
  }
  const rows = previous
    .map((release) => {
      const version = escapeHtml(versionOf(release) || release.tag_name || "");
      const published = formatDate(release.published_at || release.created_at);
      const assets = classifyAssets(release.assets);
      return `        <article class="archive-row">
          <div class="archive-meta">
            <strong>${version}</strong>
            <time datetime="${escapeHtml(release.published_at || "")}">${escapeHtml(published)}</time>
          </div>
          <div class="archive-links">${archiveLinks(assets)}</div>
        </article>`;
    })
    .join("\n");
  return `
      <section class="archive" aria-labelledby="previous-heading">
        <h2 id="previous-heading">Previous versions</h2>
        <p class="archive-note">Use these only if you need to roll back. New shops should install the latest release.</p>
${rows}
      </section>`;
}

function renderPage(releases) {
  const { latest, previous } = selectReleases(releases);
  const version = latest ? versionOf(latest) : "";
  const tag = latest?.tag_name || "";
  const assets = classifyAssets(latest?.assets);
  const versionLabel = version ? `Version ${escapeHtml(version)}` : "No public release yet";
  const status = latest
    ? `Latest release ${escapeHtml(tag)}. Windows is the supported shop target. macOS and Linux builds are unsigned convenience downloads.`
    : "Installers appear here after the next public GitHub Release. Windows is the supported shop target. macOS and Linux builds are unsigned convenience downloads.";

  return `<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <title>${PRODUCT} — Download</title>
    <meta
      name="description"
      content="Download ${PRODUCT}, the offline workshop repair manager by Moritz Alexander Wright."
    />
    <meta name="author" content="Moritz Alexander Wright" />
    <style>
      :root {
        --bg: #f4f6f8;
        --fg: #1a2332;
        --muted: #5b6b7c;
        --surface: #ffffff;
        --border: #d5dde5;
        --primary: #0f9d8a;
        --primary-fg: #ffffff;
      }
      * {
        box-sizing: border-box;
      }
      body {
        margin: 0;
        font-family: "Segoe UI", "Helvetica Neue", sans-serif;
        background: var(--bg);
        color: var(--fg);
        line-height: 1.5;
      }
      main {
        max-width: 44rem;
        margin: 0 auto;
        padding: 3rem 1.25rem 4rem;
      }
      h1 {
        margin: 0 0 0.35rem;
        font-size: 2rem;
      }
      h2 {
        margin: 2.25rem 0 0.4rem;
        font-size: 1.1rem;
      }
      .tagline {
        margin: 0 0 2rem;
        color: var(--muted);
      }
      .version {
        display: inline-block;
        margin-bottom: 1.5rem;
        padding: 0.2rem 0.6rem;
        border: 1px solid var(--border);
        border-radius: 999px;
        font-size: 0.85rem;
        color: var(--muted);
      }
      .grid {
        display: grid;
        gap: 0.75rem;
      }
      @media (min-width: 640px) {
        .grid {
          grid-template-columns: 1fr 1fr 1fr;
        }
      }
      a.card {
        display: block;
        padding: 1rem 1.1rem;
        border: 1px solid var(--border);
        border-radius: 0.75rem;
        background: var(--surface);
        color: inherit;
        text-decoration: none;
      }
      a.card:hover {
        border-color: var(--primary);
      }
      a.card.disabled {
        opacity: 0.55;
        pointer-events: none;
      }
      .os {
        font-weight: 600;
      }
      .file {
        display: block;
        margin-top: 0.25rem;
        font-size: 0.8rem;
        color: var(--muted);
        word-break: break-all;
      }
      .btn {
        display: inline-block;
        margin-top: 0.75rem;
        padding: 0.4rem 0.75rem;
        border-radius: 0.5rem;
        background: var(--primary);
        color: var(--primary-fg);
        font-size: 0.875rem;
        font-weight: 600;
      }
      .note,
      .archive-note,
      footer {
        margin-top: 2rem;
        font-size: 0.875rem;
        color: var(--muted);
      }
      .archive-note {
        margin-top: 0;
        margin-bottom: 1rem;
      }
      .archive-row {
        display: flex;
        flex-wrap: wrap;
        gap: 0.35rem 1rem;
        justify-content: space-between;
        padding: 0.75rem 0;
        border-top: 1px solid var(--border);
      }
      .archive-meta {
        display: flex;
        gap: 0.75rem;
        align-items: baseline;
      }
      .archive-meta time,
      .archive-links {
        font-size: 0.875rem;
        color: var(--muted);
      }
      .archive-links a {
        color: var(--primary);
        text-decoration: none;
        font-weight: 600;
      }
      .archive-links a:hover {
        text-decoration: underline;
      }
      footer {
        border-top: 1px solid var(--border);
        padding-top: 1rem;
      }
    </style>
  </head>
  <body>
    <main>
      <h1>${PRODUCT}</h1>
      <p class="tagline">Offline workshop repair manager.</p>
      <p class="version">${versionLabel}</p>
      <div class="grid">
        ${card("Windows", assets.windows, "10 / 11 · x64 installer")}
        ${card("macOS", assets.macos, "Disk image (.dmg)")}
        ${card("Linux", assets.linux, "AppImage")}
      </div>
      <p class="note">${status}</p>${renderArchive(previous)}
      <footer>
        Author: Moritz Alexander Wright · Copyright © 2026 · All rights reserved
      </footer>
    </main>
  </body>
</html>
`;
}

const args = parseArgs(process.argv.slice(2));
if (!args.output) {
  throw new Error(
    "Usage: node scripts/build-download-page.mjs [--input releases.json] [--repo owner/name] --output index.html",
  );
}

const html = renderPage(loadReleases(args.input, args.repo));
mkdirSync(dirname(args.output), { recursive: true });
writeFileSync(args.output, html);
console.log(`Wrote download page to ${args.output}`);
