import * as anchor from "@project-serum/anchor";

import {
  Utxo,
  Transaction,
  ADMIN_AUTH_KEYPAIR,
  REGISTERED_POOL_PDA_SPL_TOKEN,
  initLookUpTableFromFile,
  setUpMerkleTree,
  createTestAccounts,
  KEYPAIR_PRIVKEY,
  Account,
  MERKLE_TREE_KEY,
  MINT,
  offerBurnerPrivkey,
  offerAuthorityPrivkey,
  bidderPrivkey,
  feeRecipient1Privkey,
  feeRecipientPrivkey,
  TransactionParameters,
  SolMerkleTree,
  Provider as LightProvider,
  userTokenAccount,
  USER_TOKEN_ACCOUNT,
  verifierProgramTwoProgramId,
  merkleTreeProgramId,
  VerifierZero,
  updateMerkleTreeForTest,
  ADMIN_AUTH_KEY,
  VerifierTwo,
  confirmConfig,
  Relayer,
  Action,
} from "light-sdk";
import {
  Keypair as SolanaKeypair,
  SystemProgram,
  PublicKey,
} from "@solana/web3.js";
import { MockVerifier } from "../sdk/src/index";

import { buildPoseidonOpt } from "circomlibjs";
import { assert, expect } from "chai";
import { BN } from "@project-serum/anchor";
import { it } from "mocha";
const token = require("@solana/spl-token");
var POSEIDON, LOOK_UP_TABLE, KEYPAIR, relayerRecipient: PublicKey;

describe("Mock verifier functional", () => {
  // Configure the client to use the local cluster.
  process.env.ANCHOR_WALLET = process.env.HOME + "/.config/solana/id.json";
  const provider = anchor.AnchorProvider.local(
    "http://127.0.0.1:8899",
    confirmConfig,
  );
  process.env.ANCHOR_PROVIDER_URL = "http://127.0.0.1:8899";

  anchor.setProvider(provider);
  before(async () => {
    console.log("Initing accounts");

    await createTestAccounts(provider.connection, userTokenAccount);
    POSEIDON = await buildPoseidonOpt();
    KEYPAIR = new Account({
      poseidon: POSEIDON,
      seed: KEYPAIR_PRIVKEY.toString(),
    });
    await setUpMerkleTree(provider);
    relayerRecipient = SolanaKeypair.generate().publicKey;
    LOOK_UP_TABLE = await initLookUpTableFromFile(
      provider,
      "lookUpTable.txt" /*Array.from([relayerRecipient])*/,
    );
  });

  var outputUtxo;
  it("Test Deposit MockVerifier cpi VerifierTwo", async () => {
    const poseidon = await buildPoseidonOpt();

    let lightProvider = await LightProvider.native(ADMIN_AUTH_KEYPAIR);

    outputUtxo = new Utxo({
      poseidon,
      assets: [SystemProgram.programId],
      account: new Account({
        poseidon,
        seed: new Array(32).fill(1).toString(),
      }),
      amounts: [new BN(1_000_000)],
    });

    const txParams = new TransactionParameters({
      outputUtxos: [outputUtxo],
      merkleTreePubkey: MERKLE_TREE_KEY,
      sender: userTokenAccount, // just any token account
      senderFee: ADMIN_AUTH_KEY, //
      lookUpTable: LOOK_UP_TABLE,
      verifier: new VerifierTwo(),
      poseidon,
      action: Action.DEPOSIT
    });

    const appParams = {
      verifier: new MockVerifier(),
      inputs: {},
    };

    let tx = new Transaction({
      provider: lightProvider,
      params: txParams,
      appParams
    });

    await tx.compile();
    await tx.provider.provider.connection.confirmTransaction(
      await tx.provider.provider.connection.requestAirdrop(
        tx.params.accounts.authority,
        1_000_000_000,
      )
    );
    await tx.getProof();
    await tx.getAppProof();
    await tx.sendAndConfirmTransaction();
    await tx.checkBalances();
    await updateMerkleTreeForTest(provider.connection);
  });

  it("Test Withdrawal MockVerifier cpi VerifierTwo", async () => {
    const poseidon = await buildPoseidonOpt();

    let lightProvider = await LightProvider.native(ADMIN_AUTH_KEYPAIR);

    let relayer = new Relayer(
      ADMIN_AUTH_KEYPAIR.publicKey,
      lightProvider.lookUpTable,
      relayerRecipient,
      new BN(0),
    );
    await provider.connection.confirmTransaction(
      await provider.connection.requestAirdrop(relayerRecipient, 10000000),
    );

    // TODO: add check that recipients are defined if withdrawal
    const txParams = new TransactionParameters({
      inputUtxos: [outputUtxo],
      merkleTreePubkey: MERKLE_TREE_KEY,
      recipient: userTokenAccount, // just any token account
      recipientFee: SolanaKeypair.generate().publicKey, //
      verifier: new VerifierTwo(),
      action: Action.WITHDRAWAL,
      poseidon,
      relayer,
    });

    const appParams = {
      verifier: new MockVerifier(),
      inputs: {},
    };

    let tx = new Transaction({
      provider: lightProvider,
      params: txParams,
      appParams
    });

    await tx.compile();
    await tx.getProof();
    await tx.getAppProof();
    await tx.sendAndConfirmTransaction();
    await tx.checkBalances();
  });
});
