//@ts-check
import { AnchorProvider, BN, utils } from "@coral-xyz/anchor";
import {
  Connection,
  Keypair as SolanaKeypair,
  PublicKey,
  Keypair,
} from "@solana/web3.js";
import chai, { expect } from "chai";
import chaiHttp from "chai-http";
import express from "express";
import { sign } from "tweetnacl";

import {
  Provider,
  airdropSol,
  User,
  Relayer,
  useWallet,
  ADMIN_AUTH_KEYPAIR,
  createTestAccounts,
  confirmConfig,
  UserTestAssertHelper,
  Action,
  Account,
  ConfirmOptions,
} from "@lightprotocol/zk.js";
import sinon from "sinon";

import {
  updateMerkleTree,
  getIndexedTransactions,
  handleRelayRequest,
  buildMerkleTree,
  getLookUpTable,
} from "../src/services";
import { bs58 } from "@coral-xyz/anchor/dist/cjs/utils/bytes";
import { waitForBalanceUpdate } from "./test-utils/waitForBalanceUpdate";
import { AIRDROP_DECIMALS } from "../src/config";

let circomlibjs = require("circomlibjs");

chai.use(chaiHttp);
const app = express();
app.use(express.json());
app.use(express.urlencoded({ extended: false }));

// Use sinon to create a stub for the middleware
const addCorsHeadersStub = sinon
  .stub()
  .callsFake((_req: any, _res: any, next: any) => next());
app.use(addCorsHeadersStub);

app.post("/updatemerkletree", updateMerkleTree);
app.get("/lookuptable", getLookUpTable);
app.post("/relayTransaction", handleRelayRequest);
app.get("/indexedTransactions", getIndexedTransactions);
app.get("/getBuiltMerkletree", buildMerkleTree);

describe("Browser tests", () => {
  var RELAYER: Relayer;
  var poseidon: any;
  var provider: Provider;
  var user: User;
  const walletMock = useWallet(ADMIN_AUTH_KEYPAIR);
  const connection = new Connection("http://127.0.0.1:8899", "confirmed");

  before(async () => {
    poseidon = await circomlibjs.buildPoseidonOpt();

    await createTestAccounts(connection);

    const relayerRecipientSol = SolanaKeypair.generate().publicKey;
    await connection.requestAirdrop(relayerRecipientSol, 2 * AIRDROP_DECIMALS);
    let relayer = SolanaKeypair.generate();
    await airdropSol({
      connection: connection,
      lamports: 2 * AIRDROP_DECIMALS,
      recipientPublicKey: relayer.publicKey,
    });

    // TODO: This will only work as long as .env keys don't change
    RELAYER = new Relayer(
      new PublicKey("EkXDLi1APzu6oxJbg5Hnjb24kfKauJp1xCb5FAUMxf9D"),
      new PublicKey("AV3LnV78ezsEBZebNeMPtEcH1hmvSfUBC5Xbyrzqbt44"),
      new BN(100000),
    );
    await airdropSol({
      connection: connection,
      lamports: 2 * AIRDROP_DECIMALS,
      recipientPublicKey: walletMock.publicKey,
    });

    const message =
      "IMPORTANT:\nThe application will be able to spend \nyour shielded assets. \n\nOnly sign the message if you trust this\n application.\n\n View all verified integrations here: \n'https://docs.lightprotocol.com/partners'";

    const encodedMessage = utils.bytes.utf8.encode(message);
    const signature = await walletMock.signMessage(encodedMessage);
    if (
      !sign.detached.verify(
        encodedMessage,
        signature,
        walletMock.publicKey.toBytes(),
      )
    )
      throw new Error("Invalid signature!");

    if (!walletMock.signMessage) throw new Error("Wallet not connected!");
    provider = await Provider.init({
      relayer: RELAYER,
      wallet: walletMock,
      confirmConfig,
    });

    user = await User.init({
      provider,
      seed: bs58.encode(signature),
    });

    await airdropSol({
      connection: provider.provider.connection,
      recipientPublicKey: walletMock.publicKey!,
      lamports: 4 * AIRDROP_DECIMALS,
    });
  });

  it("should fail to init node feature (anchorprovider)", async () => {
    // should expect to throw
    expect(() => {
      AnchorProvider.local("http://127.0.0.1:8899", confirmConfig);
    }).to.throw("Provider local is not available on browser.");
  });

  it("(browser) should shield and update merkle tree", async () => {
    let testInputs = {
      amountSol: 15,
      token: "SOL",
      type: Action.SHIELD,
      expectedUtxoHistoryLength: 1,
    };
    const userTestAssertHelper = new UserTestAssertHelper({
      userSender: user,
      userRecipient: user,
      provider,
      testInputs,
    });
    await userTestAssertHelper.fetchAndSaveState();

    await user.shield({
      publicAmountSol: testInputs.amountSol,
      token: testInputs.token,
      confirmOptions: ConfirmOptions.spendable,
    });

    await waitForBalanceUpdate(userTestAssertHelper, user);
    await userTestAssertHelper.checkSolShielded();
  });

  it("(browser) should unshield SOL and update merkle tree", async () => {
    const solRecipient = Keypair.generate();

    const testInputs = {
      amountSol: 1,
      token: "SOL",
      type: Action.UNSHIELD,
      recipient: solRecipient.publicKey,
      expectedUtxoHistoryLength: 1,
    };

    const userTestAssertHelper = new UserTestAssertHelper({
      userSender: user,
      userRecipient: user,
      provider,
      testInputs,
    });
    // need to wait for balance update to fetch current utxos
    await user.getBalance();
    await userTestAssertHelper.fetchAndSaveState();
    await user.unshield({
      publicAmountSol: testInputs.amountSol,
      token: testInputs.token,
      recipient: testInputs.recipient,
      confirmOptions: ConfirmOptions.spendable,
    });

    await waitForBalanceUpdate(userTestAssertHelper, user);
    await userTestAssertHelper.checkSolUnshielded();
  });

  it("should transfer sol and update merkle tree ", async () => {
    const testInputs = {
      amountSol: 1,
      token: "SOL",
      type: Action.TRANSFER,
      expectedUtxoHistoryLength: 1,
      recipientSeed: bs58.encode(new Uint8Array(32).fill(9)),
      expectedRecipientUtxoLength: 1,
    };

    const recipientAccount = new Account({
      poseidon,
      seed: testInputs.recipientSeed,
    });

    const userRecipient: User = await User.init({
      provider,
      seed: testInputs.recipientSeed,
    });

    const testStateValidator = new UserTestAssertHelper({
      userSender: user,
      userRecipient,
      provider,
      testInputs,
    });
    await testStateValidator.fetchAndSaveState();
    // need to wait for balance update to fetch current utxos
    await user.getBalance();
    await user.transfer({
      amountSol: testInputs.amountSol,
      token: testInputs.token,
      recipient: recipientAccount.getPublicKey(),
    });

    await waitForBalanceUpdate(testStateValidator, user);
    await testStateValidator.checkSolTransferred();
  });
});
