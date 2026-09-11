import assert from "node:assert/strict";
import test from "node:test";
import { macosBuildEnvironment } from "./ci-macos-build.mjs";

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
