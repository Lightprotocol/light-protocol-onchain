import { expect, test } from "@oclif/test";
import { initTestEnvIfNeeded } from "../../../src/utils/initTestEnv";

describe("shield:spl sub-cli", () => {
  before(async () => {
    await initTestEnvIfNeeded();
  })
  test
  .stdout({print: true})
  .command([
    'shield:spl',
    '10',
    'USDC',
  ])
  .it("shielding 1 USDC", (ctx) => {
    expect(ctx.stdout).to.contain('Successfully shielded 10 USDC ✔');
  })

  test
  .stdout({print: true})
  .command([
    'shield:spl',
    '123', 
    'USDC',
    '-d'
  ])
  .it("shielding 1.23 USDC taking absolute input with the subcli", (ctx) => {
    expect(ctx.stdout).to.contain('Successfully shielded 1.23 USDC ✔');
  })

  test
  .stdout()
  .stderr()
  .command([
    'shield:spl',
    '10000000000000000000000000000000000000000',
    'USDC'
  ])
  .exit(2)
  .it("Should fail shield of unsufficient SPL amount")

  test
  .stdout()
  .stderr()
  .command([
    'shield:spl',
    '10',
    'USDC',
    '--recipient=TpqsASoGWfR96tVd6ePkN55S2VucK5gLjXJM2abywRU3darrKYkdYadyJsQ9vndp2khowVzuj5ZYduxxxrUun2eFAIL',
  ])
  .exit(2)
  .it("Should fail shield SPL to an invalid shielded recipient address")
})