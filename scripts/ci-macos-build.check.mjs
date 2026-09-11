import assert from "node:assert/strict";
import test from "node:test";
import {
  macosBuildEnvironment,
  notaryAuthArgs,
  resolveNotaryTimeoutMs,
  tauriBuildEnvironment,
} from "./ci-macos-build.mjs";

const complete = {
  APPLE_CERTIFICATE: "test-certificate",
  APPLE_CERTIFICATE_PASSWORD: "test-password",
  APPLE_SIGNING_IDENTITY: "Developer ID Application: Example (TEAM123456)",
  APPLE_ID: "test@example.com",
  APPLE_PASSWORD: "test-app-password",
  APPLE_TEAM_ID: "TEAM123456",
};

test("missing GitHub secrets are removed so Tauri does not import an empty certificate", () => {
  const source = Object.fromEntries(
    Object.keys(complete).map((key) => [key, ""]),
  );
  const result = macosBuildEnvironment({ ...source, PATH: "/bin" });
  assert.equal(result.signed, false);
  assert.deepEqual(result.env, { PATH: "/bin" });
  assert.equal(source.APPLE_CERTIFICATE, "");
});

test("every partial configuration fails before building and does not expose values", () => {
  for (const key of Object.keys(complete)) {
    const partial = { ...complete, [key]: "" };
    assert.throws(
      () => macosBuildEnvironment(partial),
      (error) => {
        assert.match(error.message, /Incomplete Apple signing setup/);
        assert.ok(error.message.includes(key));
        assert.ok(!error.message.includes(complete.APPLE_CERTIFICATE_PASSWORD));
        return true;
      },
    );
  }
});

test("complete Developer ID credentials enable signing and are preserved", () => {
  assert.deepEqual(macosBuildEnvironment(complete), {
    env: complete,
    signed: true,
  });
});

test("App Store identities and mismatched teams cannot sign a direct-download release", () => {
  assert.throws(
    () =>
      macosBuildEnvironment({
        ...complete,
        APPLE_SIGNING_IDENTITY: "Apple Distribution: Example (TEAM123456)",
      }),
    /Developer ID Application/,
  );
  assert.throws(
    () => macosBuildEnvironment({ ...complete, APPLE_TEAM_ID: "OTHERTEAM1" }),
    /must match/,
  );
});

test("tauri build env keeps the certificate but strips notarization secrets", () => {
  const { env, signed } = macosBuildEnvironment(complete);
  const buildEnv = tauriBuildEnvironment(env, signed);
  assert.equal(buildEnv.APPLE_CERTIFICATE, complete.APPLE_CERTIFICATE);
  assert.equal(
    buildEnv.APPLE_CERTIFICATE_PASSWORD,
    complete.APPLE_CERTIFICATE_PASSWORD,
  );
  assert.equal(
    buildEnv.APPLE_SIGNING_IDENTITY,
    complete.APPLE_SIGNING_IDENTITY,
  );
  assert.equal(buildEnv.APPLE_ID, undefined);
  assert.equal(buildEnv.APPLE_PASSWORD, undefined);
  assert.equal(buildEnv.APPLE_TEAM_ID, undefined);
});

test("notary auth args and timeout helper stay explicit for CI", () => {
  assert.deepEqual(notaryAuthArgs(complete), [
    "--apple-id",
    complete.APPLE_ID,
    "--password",
    complete.APPLE_PASSWORD,
    "--team-id",
    complete.APPLE_TEAM_ID,
  ]);
  assert.equal(resolveNotaryTimeoutMs({}), 45 * 60 * 1000);
  assert.equal(
    resolveNotaryTimeoutMs({ APPLE_NOTARY_TIMEOUT_MS: "900000" }),
    900_000,
  );
  assert.throws(
    () => resolveNotaryTimeoutMs({ APPLE_NOTARY_TIMEOUT_MS: "1000" }),
    /APPLE_NOTARY_TIMEOUT_MS/,
  );
});
