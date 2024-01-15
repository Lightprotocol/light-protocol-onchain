import { expect, test } from "@oclif/test";
import { initTestEnvIfNeeded } from "../../../src";

describe("unshield:sol", () => {
  before(async () => {
    await initTestEnvIfNeeded();
  });
  test
    .stdout()
    .command([
      "unshield:sol",
      "0.2",
      "E2CDgD4vq636mLf9pgMTyKdK3k8gbPZM95YetYMfPLbc",
      "--localTestRpc",
    ])
    .it("Unshielding 0.2 SOL", async (ctx) => {
      expect(ctx.stdout).to.contain("Successfully decompressed 0.2 SOL ✔");
    });

  test
    .stdout()
    .stderr()
    .command([
      "unshield:sol",
      "300000",
      "E2CDgD4vq636mLf9pgMTyKdK3k8gbPZM95YetYMfPLbc",
      "--localTestRpc",
    ])
    .exit(2)
    .it("Should fail unshield of unsufficient SOL amount");
});
