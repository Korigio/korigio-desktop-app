import { spawnSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const signingKeys = [
  "APPLE_CERTIFICATE",
  "APPLE_CERTIFICATE_PASSWORD",
  "APPLE_SIGNING_IDENTITY",
  "APPLE_ID",
  "APPLE_PASSWORD",
  "APPLE_TEAM_ID",
];

/** Keys that make Tauri auto-notarize (unbounded `--wait`). Kept for our step. */
const notarizationKeys = ["APPLE_ID", "APPLE_PASSWORD", "APPLE_TEAM_ID"];

const DEFAULT_NOTARY_TIMEOUT_MS = 45 * 60 * 1000;
const DEFAULT_NOTARY_POLL_MS = 30_000;

// GitHub supplies missing secrets as empty strings; Tauri checks presence.
export function macosBuildEnvironment(source) {
  const env = { ...source };
  const configured = signingKeys.filter((key) => env[key]?.trim());
  if (configured.length > 0 && configured.length !== signingKeys.length) {
    const missing = signingKeys.filter((key) => !configured.includes(key));
    throw new Error(
      `Incomplete Apple signing setup. Missing: ${missing.join(", ")}`,
    );
  }
  const signed = configured.length > 0;
  if (signed) {
    env.APPLE_SIGNING_IDENTITY = env.APPLE_SIGNING_IDENTITY.trim();
    if (!env.APPLE_SIGNING_IDENTITY.startsWith("Developer ID Application: ")) {
      throw new Error(
        "Direct-download releases need a Developer ID Application identity.",
      );
    }
    if (!env.APPLE_SIGNING_IDENTITY.endsWith(`(${env.APPLE_TEAM_ID.trim()})`)) {
      throw new Error("APPLE_TEAM_ID must match the signing identity's team.");
    }
    env.APPLE_TEAM_ID = env.APPLE_TEAM_ID.trim();
    env.APPLE_ID = env.APPLE_ID.trim();
    env.APPLE_PASSWORD = env.APPLE_PASSWORD.trim();
  } else {
    for (const key of signingKeys) delete env[key];
  }
  return { env, signed };
}

/** Tauri signs from APPLE_CERTIFICATE*; strip notarization env so it cannot hang. */
export function tauriBuildEnvironment(env, signed) {
  const buildEnv = { ...env };
  if (signed) {
    for (const key of notarizationKeys) delete buildEnv[key];
  }
  return buildEnv;
}

export function notaryAuthArgs(env) {
  return [
    "--apple-id",
    env.APPLE_ID,
    "--password",
    env.APPLE_PASSWORD,
    "--team-id",
    env.APPLE_TEAM_ID,
  ];
}

function run(command, args, options = {}) {
  const result = spawnSync(command, args, { stdio: "inherit", ...options });
  if (result.error) throw result.error;
  if (result.status !== 0)
    throw new Error(`${command} failed (${result.status}).`);
  return result;
}

function runCapture(command, args, options = {}) {
  const result = spawnSync(command, args, {
    encoding: "utf8",
    ...options,
  });
  if (result.error) throw result.error;
  return result;
}

function parseJsonOutput(stdout, label) {
  const text = (stdout ?? "").trim();
  if (!text) throw new Error(`${label} produced no output.`);
  try {
    return JSON.parse(text);
  } catch {
    // notarytool sometimes prints warnings before JSON.
    const start = text.indexOf("{");
    const end = text.lastIndexOf("}");
    if (start >= 0 && end > start) {
      return JSON.parse(text.slice(start, end + 1));
    }
    throw new Error(`Failed to parse ${label} JSON.`);
  }
}

export function resolveNotaryTimeoutMs(source = process.env) {
  const raw = source.APPLE_NOTARY_TIMEOUT_MS?.trim();
  if (!raw) return DEFAULT_NOTARY_TIMEOUT_MS;
  const value = Number(raw);
  if (!Number.isFinite(value) || value < 60_000) {
    throw new Error(
      "APPLE_NOTARY_TIMEOUT_MS must be a number of milliseconds >= 60000.",
    );
  }
  return value;
}

export function preflightNotaryCredentials(env) {
  console.log("Checking Apple notarization credentials…");
  const result = runCapture(
    "xcrun",
    [
      "notarytool",
      "history",
      ...notaryAuthArgs(env),
      "--output-format",
      "json",
    ],
    { env },
  );
  if (result.status !== 0) {
    const detail = [result.stderr, result.stdout]
      .filter(Boolean)
      .join("\n")
      .trim();
    throw new Error(
      `Apple notarization credentials failed preflight. Use an app-specific password (not the account password) and matching APPLE_TEAM_ID.${detail ? `\n${detail}` : ""}`,
    );
  }
  console.log("Apple notarization credentials accepted.");
}

function findDmg(cwd, productName) {
  const bundleDir = resolve(cwd, "src-tauri/target/release/bundle/dmg");
  const listing = runCapture(
    "bash",
    ["-lc", `ls -1 "${bundleDir}"/*.dmg 2>/dev/null || true`],
    {
      env: process.env,
    },
  );
  const dmgs = (listing.stdout ?? "")
    .split("\n")
    .map((line) => line.trim())
    .filter(Boolean);
  if (dmgs.length === 0) {
    throw new Error(`No DMG found under ${bundleDir}.`);
  }
  const preferred = dmgs.find((path) =>
    path.toLowerCase().includes(productName.toLowerCase()),
  );
  return preferred ?? dmgs[0];
}

function sleepSync(ms) {
  spawnSync("sleep", [String(Math.ceil(ms / 1000))], { stdio: "ignore" });
}

export function notarizeAndStapleDmg(dmgPath, env, options = {}) {
  const timeoutMs = options.timeoutMs ?? resolveNotaryTimeoutMs(env);
  const pollMs = options.pollMs ?? DEFAULT_NOTARY_POLL_MS;
  const auth = notaryAuthArgs(env);

  console.log(`Submitting ${dmgPath} to Apple notarization…`);
  const submit = runCapture(
    "xcrun",
    ["notarytool", "submit", dmgPath, ...auth, "--output-format", "json"],
    { env },
  );
  if (submit.status !== 0) {
    const detail = [submit.stderr, submit.stdout]
      .filter(Boolean)
      .join("\n")
      .trim();
    throw new Error(`notarytool submit failed.${detail ? `\n${detail}` : ""}`);
  }

  const submitted = parseJsonOutput(submit.stdout, "notarytool submit");
  const submissionId = submitted.id;
  if (!submissionId) {
    throw new Error("notarytool submit did not return a submission id.");
  }
  console.log(`Notarization submission id: ${submissionId}`);

  const started = Date.now();
  let lastStatus = "Submitted";
  while (Date.now() - started < timeoutMs) {
    const info = runCapture(
      "xcrun",
      ["notarytool", "info", submissionId, ...auth, "--output-format", "json"],
      { env },
    );
    if (info.status === 0) {
      const payload = parseJsonOutput(info.stdout, "notarytool info");
      lastStatus = payload.status ?? "Unknown";
      const elapsedMin = ((Date.now() - started) / 60_000).toFixed(1);
      console.log(`[${elapsedMin}m] notarization status: ${lastStatus}`);
      if (lastStatus === "Accepted") {
        console.log("Stapling notarization ticket to DMG…");
        run("xcrun", ["stapler", "staple", dmgPath], { env });
        return { submissionId, status: lastStatus };
      }
      if (lastStatus === "Invalid" || lastStatus === "Rejected") {
        const log = runCapture(
          "xcrun",
          ["notarytool", "log", submissionId, ...auth],
          { env },
        );
        const detail = [log.stdout, log.stderr]
          .filter(Boolean)
          .join("\n")
          .trim();
        throw new Error(
          `Apple notarization ${lastStatus} for ${submissionId}.${detail ? `\n${detail}` : ""}`,
        );
      }
    } else {
      console.warn(
        `notarytool info failed (will retry): ${(info.stderr || info.stdout || "").trim()}`,
      );
    }
    sleepSync(pollMs);
  }

  throw new Error(
    [
      `Apple notarization still "${lastStatus}" after ${Math.round(timeoutMs / 60_000)} minutes.`,
      `Submission id: ${submissionId}`,
      "First Developer ID submissions are often held by Apple for hours (sometimes 24h+).",
      "Do not keep re-submitting — that queues more jobs. Check status with:",
      `  xcrun notarytool info ${submissionId} --apple-id … --password … --team-id …`,
      "When status is Accepted, re-run the release workflow (or staple the DMG locally).",
    ].join("\n"),
  );
}

function main() {
  if (process.platform !== "darwin")
    throw new Error("macOS builds must run on macOS.");
  const cwd = resolve(dirname(fileURLToPath(import.meta.url)), "..");
  const { env, signed } = macosBuildEnvironment(process.env);
  console.log(
    signed
      ? "Building with Developer ID signing; notarization runs after the DMG with a bounded wait."
      : "Apple signing secrets are absent; building an unsigned convenience DMG.",
  );

  if (signed) {
    preflightNotaryCredentials(env);
  }

  run("npm", ["run", "tauri", "build", "--", "--bundles", "dmg"], {
    cwd,
    env: tauriBuildEnvironment(env, signed),
  });

  if (signed) {
    // `--bundles dmg` signs the DMG then removes the intermediate .app ("Cleaning …").
    // Verify / notarize / staple the DMG only — that is the uploaded artifact.
    const config = JSON.parse(
      readFileSync(resolve(cwd, "src-tauri/tauri.conf.json"), "utf8"),
    );
    const dmg = findDmg(cwd, config.productName);
    run("codesign", ["--verify", "--verbose=2", dmg], { cwd, env });
    notarizeAndStapleDmg(dmg, env);
    run("xcrun", ["stapler", "validate", dmg], { cwd, env });
    run(
      "spctl",
      [
        "--assess",
        "--type",
        "open",
        "--context",
        "context:primary-signature",
        "--verbose=2",
        dmg,
      ],
      { cwd, env },
    );
  }
}

if (
  process.argv[1] &&
  resolve(process.argv[1]) === fileURLToPath(import.meta.url)
) {
  try {
    main();
  } catch (error) {
    console.error(error.message);
    process.exitCode = 1;
  }
}
