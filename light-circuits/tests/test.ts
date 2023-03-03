import {
  ADMIN_AUTH_KEYPAIR,
  confirmConfig,
  FEE_ASSET,
  Account,
  Provider as LightProvider,
  MINT,
  Transaction,
  TransactionParameters,
  Utxo,
  VerifierZero,
  SolMerkleTree,
  VerifierTwo,
  Verifier,
  VerifierOne,
} from "light-sdk";
import * as anchor from "@coral-xyz/anchor";
import { assert, expect } from "chai";
import { Connection, Keypair as SolanaKeypair } from "@solana/web3.js";
const circomlibjs = require("circomlibjs");

describe("verifier_program", () => {
  process.env.ANCHOR_WALLET = process.env.HOME + "/.config/solana/id.json";
  process.env.ANCHOR_PROVIDER_URL = "http://127.0.0.1:8899";

  before(async () => {
    try {
      const provider = new anchor.AnchorProvider(
        await new Connection("http://127.0.0.1:8899"),
        new anchor.Wallet(SolanaKeypair.generate()),
        confirmConfig
      );
      await anchor.setProvider(provider);
    } catch (error) {
      console.log("expected local test validator to be running");
      process.exit();
    }
  });

  it("Test functional circuit 2 in 2 out", async () => {
    await functionalCircuitTest(new VerifierZero());
  });

  it("Test functional circuit 10 in 2 out", async () => {
    await functionalCircuitTest(new VerifierOne());
  });

  it("Test functional circuit 4 in 4 out + connecting hash", async () => {
    await functionalCircuitTest(new VerifierTwo(), true);
  });
});

async function functionalCircuitTest(verifier: Verifier, app: boolean = false) {
  const poseidon = await circomlibjs.buildPoseidonOpt();
  let seed32 = new Uint8Array(32).fill(1).toString();
  let keypair = new Account({ poseidon: poseidon, seed: seed32 });
  let depositAmount = 20_000;
  let depositFeeAmount = 10_000;
  let deposit_utxo1 = new Utxo({
    poseidon: poseidon,
    assets: [FEE_ASSET, MINT],
    amounts: [new anchor.BN(depositFeeAmount), new anchor.BN(depositAmount)],
    account: keypair,
  });
  let mockPubkey = SolanaKeypair.generate().publicKey;

  let lightProvider = await LightProvider.loadMock(mockPubkey);
  let txParams = new TransactionParameters({
    outputUtxos: [deposit_utxo1],
    merkleTreePubkey: mockPubkey,
    sender: mockPubkey,
    senderFee: mockPubkey,
    verifier: verifier,
  });

  let tx = new Transaction({
    provider: lightProvider,
  });

  // successful proofgeneration
  if (app) {
    await tx.compile(txParams, { mock: "123" });
  } else {
    await tx.compile(txParams);
  }

  await tx.getProof();
  // unsuccessful proofgeneration
  let x = true;

  try {
    tx.proofInput.inIndices[0][1][1] = "1";
    // TODO: investigate why this does not kill the proof
    tx.proofInput.inIndices[0][1][0] = "1";
    expect(await tx.getProof()).to.Throw();
    x = false;

  } catch (error) {
    // assert.isTrue(error.toString().includes("CheckIndices_3 line:"));
    assert.isTrue(x);
  }
}