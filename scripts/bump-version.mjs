#!/usr/bin/env node
/**
 * Keep package.json, tauri.conf.json, and Cargo.toml on the same semver.
 *
 *   node scripts/bump-version.mjs 1.0.0
 *   node scripts/bump-version.mjs patch|minor|major
 *   node scripts/bump-version.mjs --check
 *   node scripts/bump-version.mjs --check v1.0.0
 */
import { execFileSync } from "node:child_process";
import { readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const packageJsonPath = join(root, "package.json");
const tauriConfPath = join(root, "src-tauri/tauri.conf.json");
const cargoTomlPath = join(root, "src-tauri/Cargo.toml");
const cargoLockPath = join(root, "src-tauri/Cargo.lock");

const SEMVER = /^(\d+)\.(\d+)\.(\d+)$/;

function readJson(path) {
  return JSON.parse(readFileSync(path, "utf8"));
}

function currentVersion() {
  const fromTauri = readJson(tauriConfPath).version;
  if (typeof fromTauri !== "string" || !SEMVER.test(fromTauri)) {
    throw new Error(`Invalid version in tauri.conf.json: ${fromTauri}`);
  }
  return fromTauri;
}

function bump(from, kind) {
  const match = SEMVER.exec(from);
  if (!match) {
    throw new Error(`Not a patch.minor.major version: ${from}`);
  }
  let [, major, minor, patch] = match.map((part, index) =>
    index === 0 ? part : Number(part),
  );
  if (kind === "major") {
    major += 1;
    minor = 0;
    patch = 0;
  } else if (kind === "minor") {
    minor += 1;
    patch = 0;
  } else if (kind === "patch") {
    patch += 1;
  } else {
    throw new Error(`Unknown bump kind: ${kind}`);
  }
  return `${major}.${minor}.${patch}`;
}

function versionsInSync(expected) {
  const pkg = readJson(packageJsonPath).version;
  const tauri = readJson(tauriConfPath).version;
  const cargo = /(?:^|\n)version = "([^"]+)"/.exec(
    readFileSync(cargoTomlPath, "utf8"),
  )?.[1];
  const lock = /name = "repair-manager"\nversion = "([^"]+)"/.exec(
    readFileSync(cargoLockPath, "utf8"),
  )?.[1];
  const mismatches = [];
  if (pkg !== expected) {
    mismatches.push(`package.json=${pkg}`);
  }
  if (tauri !== expected) {
    mismatches.push(`tauri.conf.json=${tauri}`);
  }
  if (cargo !== expected) {
    mismatches.push(`Cargo.toml=${cargo}`);
  }
  if (lock !== expected) {
    mismatches.push(`Cargo.lock=${lock}`);
  }
  return mismatches;
}

function writeVersion(next) {
  const pkg = readJson(packageJsonPath);
  pkg.version = next;
  writeFileSync(packageJsonPath, `${JSON.stringify(pkg, null, 2)}\n`);

  const tauri = readJson(tauriConfPath);
  tauri.version = next;
  writeFileSync(tauriConfPath, `${JSON.stringify(tauri, null, 2)}\n`);

  const cargoToml = readFileSync(cargoTomlPath, "utf8").replace(
    /^version = "[^"]+"/m,
    `version = "${next}"`,
  );
  writeFileSync(cargoTomlPath, cargoToml);

  const cargoLock = readFileSync(cargoLockPath, "utf8").replace(
    /(name = "repair-manager"\nversion = ")[^"]+"/,
    `$1${next}"`,
  );
  writeFileSync(cargoLockPath, cargoLock);

  execFileSync("npm", ["install", "--package-lock-only", "--ignore-scripts"], {
    cwd: root,
    stdio: "inherit",
  });
}

function stripV(value) {
  return value.startsWith("v") ? value.slice(1) : value;
}

const args = process.argv.slice(2);
const check = args[0] === "--check";
const checkAgainst = check ? args[1] : undefined;
const bumpArg = check ? undefined : args[0];

if (check) {
  const expected = checkAgainst ? stripV(checkAgainst) : currentVersion();
  if (!SEMVER.test(expected)) {
    throw new Error(`Not a valid version to check: ${expected}`);
  }
  const mismatches = versionsInSync(expected);
  if (mismatches.length > 0) {
    console.error(
      `Version mismatch (expected ${expected}): ${mismatches.join(", ")}`,
    );
    process.exit(1);
  }
  console.log(`Version ${expected} is in sync.`);
  process.exit(0);
}

if (!bumpArg) {
  console.error(
    "Usage: node scripts/bump-version.mjs <1.2.3|patch|minor|major>\n       node scripts/bump-version.mjs --check [v1.2.3]",
  );
  process.exit(1);
}

const from = currentVersion();
const next = SEMVER.test(bumpArg) ? bumpArg : bump(from, bumpArg);
if (!SEMVER.test(next)) {
  throw new Error(`Refusing to write invalid version: ${next}`);
}
writeVersion(next);
console.log(`Bumped ${from} → ${next}`);
