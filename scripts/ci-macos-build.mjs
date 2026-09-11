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
  } else {
    for (const key of signingKeys) delete env[key];
  }
  return { env, signed };
}

function run(command, args, options) {
  const result = spawnSync(command, args, { stdio: "inherit", ...options });
  if (result.error) throw result.error;
  if (result.status !== 0)
    throw new Error(`${command} failed (${result.status}).`);
}

function main() {
  if (process.platform !== "darwin")
    throw new Error("macOS builds must run on macOS.");
  const cwd = resolve(dirname(fileURLToPath(import.meta.url)), "..");
  const { env, signed } = macosBuildEnvironment(process.env);
  console.log(
    signed
      ? "Building with Developer ID signing and Apple notarization."
      : "Apple signing secrets are absent; building an unsigned convenience DMG.",
  );
  run("npm", ["run", "tauri", "build", "--", "--bundles", "dmg"], { cwd, env });
  if (signed) {
    const config = JSON.parse(
      readFileSync(resolve(cwd, "src-tauri/tauri.conf.json"), "utf8"),
    );
    const app = resolve(
      cwd,
      "src-tauri/target/release/bundle/macos",
      `${config.productName}.app`,
    );
    run("codesign", ["--verify", "--deep", "--strict", "--verbose=2", app], {
      cwd,
      env,
    });
    run("xcrun", ["stapler", "validate", app], { cwd, env });
    run("spctl", ["--assess", "--type", "execute", "--verbose=2", app], {
      cwd,
      env,
    });
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
