import * as anchor from "@coral-xyz/anchor";
import {
  Keypair as SolanaKeypair,
  PublicKey,
  SystemProgram,
} from "@solana/web3.js";
import _ from "lodash";
import { assert } from "chai";
const chai = require("chai");
const chaiAsPromised = require("chai-as-promised");
// init chai-as-promised support
chai.use(chaiAsPromised);

let circomlibjs = require("circomlibjs");

// TODO: add and use  namespaces in SDK
import {
  Utxo,
  setUpMerkleTree,
  initLookUpTableFromFile,
  merkleTreeProgramId,
  ADMIN_AUTH_KEYPAIR,
  MINT,
  Provider,
  newAccountWithTokens,
  createTestAccounts,
  confirmConfig,
  Relayer,
  User,
  strToArr,
  TOKEN_REGISTRY,
  Account,
  CreateUtxoErrorCode,
  UserErrorCode,
  TransactionErrorCode,
  ADMIN_AUTH_KEY,
  TestRelayer,
  fetchNullifierAccountInfo,
  Action,
  Balance,
  TokenUtxoBalance,
} from "light-sdk";

import { BN, AnchorProvider } from "@coral-xyz/anchor";
import { getAssociatedTokenAddressSync } from "@solana/spl-token";
import { bs58 } from "@coral-xyz/anchor/dist/cjs/utils/bytes";

var LOOK_UP_TABLE;
var POSEIDON;
var RELAYER;

// TODO: remove deprecated function calls
describe("Test User", () => {
  // Configure the client to use the local cluster.
  process.env.ANCHOR_WALLET = process.env.HOME + "/.config/solana/id.json";
  process.env.ANCHOR_PROVIDER_URL = "http://127.0.0.1:8899";

  const provider = AnchorProvider.local("http://127.0.0.1:8899", confirmConfig);
  anchor.setProvider(provider);
  console.log("merkleTreeProgram: ", merkleTreeProgramId.toBase58());

  const userKeypair = ADMIN_AUTH_KEYPAIR; //new SolanaKeypair();

  before("init test setup Merkle tree lookup table etc ", async () => {
    let initLog = console.log;
    // console.log = () => {};
    await createTestAccounts(provider.connection);
    LOOK_UP_TABLE = await initLookUpTableFromFile(provider);
    await setUpMerkleTree(provider);
    // console.log = initLog;
    POSEIDON = await circomlibjs.buildPoseidonOpt();

    const relayerRecipientSol = SolanaKeypair.generate().publicKey;

    await provider.connection.requestAirdrop(
      relayerRecipientSol,
      2_000_000_000,
    );

    RELAYER = await new TestRelayer(
      userKeypair.publicKey,
      LOOK_UP_TABLE,
      relayerRecipientSol,
      new BN(100000),
    );
  });

  it.only("(user class) shield SPL", async () => {
    let amount = 20;
    let token = "USDC";
    console.log("test user wallet: ", userKeypair.publicKey.toBase58());
    const provider = await Provider.init({
      wallet: userKeypair,
      relayer: RELAYER,
    }); // userKeypair
    let res = await provider.provider.connection.requestAirdrop(
      userKeypair.publicKey,
      2_000_000_000,
    );
    // get token
    const tokenCtx = TOKEN_REGISTRY.get(token);

    const userSplAccount = getAssociatedTokenAddressSync(
      tokenCtx!.mint,
      ADMIN_AUTH_KEYPAIR.publicKey,
    );
    const preTokenBalance =
      await provider.provider.connection.getTokenAccountBalance(userSplAccount);

    await provider.provider.connection.confirmTransaction(res, "confirmed");

    const user: User = await User.init({ provider });
    const preShieldedBalance = await user.getBalance();

    await user.shield({ publicAmountSpl: amount, token });

    // TODO: add random amount and amount checks
    await user.provider.latestMerkleTree();
    let balance: Balance;
    try {
      balance = await user.getBalance();
    } catch (e) {
      console.log(e);

      throw new Error(`ayayay ${e}`);
    }
    let tokenBalanceAfter: TokenUtxoBalance = balance.tokenBalances.get(tokenCtx?.mint.toBase58());
    let tokenBalancePre = preShieldedBalance.tokenBalances.get(tokenCtx?.mint.toBase58());
    console.log(balance);
    
    // assert that the user's shielded balance has increased by the amount shielded
    assert.equal(
      tokenBalanceAfter.totalBalanceSpl.toNumber(),
        amount * tokenCtx?.decimals.toNumber(),
      `shielded balance after ${tokenBalanceAfter.totalBalanceSpl.toNumber()} != shield amount ${
        amount * tokenCtx?.decimals.toNumber()
      }`,
    );

    // assert that the user's token balance has decreased by the amount shielded
    const postTokenBalance =
      await provider.provider.connection.getTokenAccountBalance(userSplAccount);
    assert.equal(
      postTokenBalance.value.uiAmount,
      preTokenBalance.value.uiAmount - amount,
      `user token balance after ${postTokenBalance.value.uiAmount} != user token balance before ${preTokenBalance.value.uiAmount} - shield amount ${amount}`,
    );

    // assert that the user's sol shielded balance has increased by the additional sol amount
    // let solBalanceAfter = balance.find(
    //   (b) => b.tokenAccount.toBase58() === SystemProgram.programId.toString(),
    // );
    // let solBalancePre = preShieldedBalance.find(
    //   (b) => b.tokenAccount.toBase58() === SystemProgram.programId.toString(),
    // );
    // console.log("solBalancePre", solBalancePre);
    // console.log("solBalanceAfter", solBalanceAfter);
    assert.equal(
      tokenBalanceAfter.totalBalanceSol.toNumber(),
      150000, //+ 2 * 1e9, this MINIMZM
      `shielded sol balance after ${tokenBalanceAfter.totalBalanceSol.toNumber()} != shield amount 0//2 aka min sol amount (50k)`,
    );

    assert.equal(tokenBalanceAfter.spentUtxos.size, 0);
    console.log(tokenBalanceAfter.utxos.entries().next().value);


    assert.notEqual(
      fetchNullifierAccountInfo(tokenBalanceAfter.utxos.entries().next().value[1]._nullifier, provider.connection),
      null,
    );

    const indexedTransactions = await provider.relayer.getIndexedTransactions(
      provider.provider.connection,
    );

    const recentTransaction = indexedTransactions[0];
    assert.equal(indexedTransactions.length, 1);
    assert.equal(
      recentTransaction.amountSpl.div(tokenCtx.decimals).toNumber(),
      amount,
    );
    assert.equal(
      recentTransaction.from.toBase58(),
      provider.wallet.publicKey.toBase58(),
    );
    assert.equal(recentTransaction.commitment, tokenBalanceAfter.utxos.entries().next().value[1]._commitment);
    assert.equal(recentTransaction.type, Action.SHIELD);
    assert.equal(recentTransaction.relayerFee.toString(), "0");
    assert.equal(
      recentTransaction.relayerRecipientSol.toBase58(),
      PublicKey.default.toBase58(),
    );
  });

  it("(user class) shield SOL", async () => {
    let amount = 15;
    let token = "SOL";
    const provider = await Provider.init({
      wallet: userKeypair,
      relayer: RELAYER,
    }); // userKeypair
    let res = await provider.provider.connection.requestAirdrop(
      userKeypair.publicKey,
      4_000_000_000,
    );
    await provider.provider.connection.confirmTransaction(res, "confirmed");
    const user: User = await User.init({ provider });
    const tokenCtx = TOKEN_REGISTRY.get(token);
    const preShieldedBalance = await user.getBalance();
    const preSolBalance = await provider.provider.connection.getBalance(
      userKeypair.publicKey,
    );
    const previousUtxos = user.utxos;

    await user.shield({ publicAmountSol: amount, token });
    // TODO: add random amount and amount checks
    await user.provider.latestMerkleTree();

    let balance = await user.getBalance();
    let solShieldedBalanceAfter = balance.find(
      (b) => b.tokenAccount.toBase58() === tokenCtx?.mint.toBase58(),
    );
    let solShieldedBalancePre = preShieldedBalance.find(
      (b) => b.tokenAccount.toBase58() === tokenCtx?.mint.toBase58(),
    );

    // console.log("solShieldedBalanceAfter", solShieldedBalanceAfter);
    // console.log("solShieldedBalancePre", solShieldedBalancePre);

    // assert that the user's token balance has decreased by the amount shielded
    const postSolBalance = await provider.provider.connection.getBalance(
      userKeypair.publicKey,
    );
    let tempAccountCost = 3502840 - 1255000; //x-y nasty af. underterministic: costs more(y) if shielded SPL before!
    // console.log("postSolBalance", postSolBalance);
    // console.log("preSolBalance", preSolBalance);
    // assert that the user's shielded balance has increased by the amount shielded
    assert.equal(
      solShieldedBalanceAfter.amount.toNumber(),
      solShieldedBalancePre.amount.toNumber() +
        amount * tokenCtx?.decimals.toNumber(),
      `shielded balance after ${
        solShieldedBalanceAfter.amount
      } != shield amount ${amount * tokenCtx?.decimals.toNumber()}`,
    );

    assert.equal(
      postSolBalance,
      preSolBalance - amount * tokenCtx.decimals.toNumber() + tempAccountCost,
      `user token balance after ${postSolBalance} != user token balance before ${preSolBalance} - shield amount ${amount} sol + tempAccountCost! ${tempAccountCost}`,
    );

    let commitmentIndex = user.spentUtxos.findIndex(
      (utxo) =>
        utxo.getCommitment(POSEIDON) === user.utxos[0].getCommitment(POSEIDON),
    );

    let commitmentSpent = user.utxos.findIndex(
      (utxo) =>
        utxo.getCommitment(POSEIDON) ===
        previousUtxos[0].getCommitment(POSEIDON),
    );

    assert.notEqual(
      fetchNullifierAccountInfo(user.utxos[0]._nullifier, provider.connection),
      null,
    );
    assert.equal(user.spentUtxos.length, 1);
    assert.equal(user.spentUtxos[0].amounts[0].toNumber(), 150000);
    assert.equal(user.utxos.length, 1);
    assert.equal(commitmentIndex, -1);
    assert.equal(commitmentSpent, -1);

    const indexedTransactions = await provider.relayer.getIndexedTransactions(
      provider.provider.connection,
    );
    const recentTransaction = indexedTransactions[0];
    assert.equal(indexedTransactions.length, 2);
    assert.equal(
      recentTransaction.amountSol.div(tokenCtx.decimals).toNumber(),
      amount,
    );
    assert.equal(
      recentTransaction.from.toBase58(),
      provider.wallet.publicKey.toBase58(),
    );
    assert.equal(recentTransaction.commitment, user.utxos[0]._commitment);
    assert.equal(recentTransaction.type, Action.SHIELD);
    assert.equal(recentTransaction.relayerFee.toString(), "0");
    assert.equal(
      recentTransaction.relayerRecipientSol.toBase58(),
      PublicKey.default.toBase58(),
    );
  });

  it("(user class) unshield SPL", async () => {
    let amount = 1;
    let token = "USDC";
    let solRecipient = SolanaKeypair.generate();
    const provider = await Provider.init({
      wallet: userKeypair,
      relayer: RELAYER,
    }); // userKeypair
    let res = await provider.provider.connection.requestAirdrop(
      userKeypair.publicKey,
      2_000_000_000,
    );

    const tokenCtx = TOKEN_REGISTRY.get(token);

    const recipientSplBalance = getAssociatedTokenAddressSync(
      tokenCtx!.mint,
      solRecipient.publicKey,
    );

    await provider.provider.connection.confirmTransaction(res, "confirmed");
    let preTokenBalance = { value: { uiAmount: 0 } };
    // TODO: add test case for if recipient doesnt have account yet -> relayer must create it
    await newAccountWithTokens({
      connection: provider.provider.connection,
      MINT,
      ADMIN_AUTH_KEYPAIR: userKeypair,
      userAccount: solRecipient,
      amount: new anchor.BN(0),
    });
    try {
      preTokenBalance =
        await provider.provider.connection.getTokenAccountBalance(
          recipientSplBalance,
        );
    } catch (e) {
      console.log(
        "recipient account does not exist yet (creating as part of user class)",
      );
    }

    const user: User = await User.init({ provider });
    const preShieldedBalance = await user.getBalance();
    const previousUtxos = user.utxos;

    await user.unshield({
      publicAmountSpl: amount,
      token,
      recipientSpl: solRecipient.publicKey,
    });

    await user.provider.latestMerkleTree();

    let balance = await user.getBalance();
    let tokenBalanceAfter = balance.find(
      (b) => b.tokenAccount.toBase58() === tokenCtx?.mint.toBase58(),
    );
    let tokenBalancePre = preShieldedBalance.find(
      (b) => b.tokenAccount.toBase58() === tokenCtx?.mint.toBase58(),
    );

    // assert that the user's shielded balance has decreased by the amount unshielded
    assert.equal(
      tokenBalanceAfter.amount.toNumber(),
      tokenBalancePre.amount.toNumber() -
        amount * tokenCtx?.decimals.toNumber(), // TODO: check that fees go ?
      `shielded balance after ${tokenBalanceAfter.amount} != unshield amount ${
        amount * tokenCtx?.decimals.toNumber()
      }`,
    );

    // assert that the user's token balance has decreased by the amount shielded
    const postTokenBalance =
      await provider.provider.connection.getTokenAccountBalance(
        recipientSplBalance,
        "confirmed",
      );

    assert.equal(
      postTokenBalance.value.uiAmount,
      preTokenBalance.value.uiAmount + amount,
      `user token balance after ${postTokenBalance.value.uiAmount} != user token balance before ${preTokenBalance.value.uiAmount} + unshield amount ${amount}`,
    );

    // assert that the user's sol shielded balance has increased by the additional sol amount
    let solBalanceAfter = balance.find(
      (b) => b.tokenAccount.toBase58() === SystemProgram.programId.toString(),
    );
    let solBalancePre = preShieldedBalance.find(
      (b) => b.tokenAccount.toBase58() === SystemProgram.programId.toString(),
    );
    const minimumBalance = 150000;
    const tokenAccountFee = 500_000;
    assert.equal(
      solBalanceAfter.amount.toNumber(),
      solBalancePre.amount.toNumber() - minimumBalance - tokenAccountFee, // FIXME: no fees being charged here apparently
      `shielded sol balance after ${
        solBalanceAfter.amount
      } != unshield amount ${
        solBalancePre.amount.toNumber() - minimumBalance - tokenAccountFee
      }`,
    );

    assert.notEqual(
      fetchNullifierAccountInfo(user.utxos[0]._nullifier, provider.connection),
      null,
    );

    let commitmentIndex = user.spentUtxos.findIndex(
      (utxo) => utxo._commitment === user.utxos[0]._commitment,
    );

    let commitmentSpent = user.utxos.findIndex(
      (utxo) => utxo._commitment === previousUtxos[0]._commitment,
    );

    assert.equal(user.spentUtxos.length, 2);
    assert.equal(user.utxos.length, 1);
    assert.equal(commitmentIndex, -1);
    assert.equal(commitmentSpent, -1);

    const indexedTransactions = await provider.relayer.getIndexedTransactions(
      provider.provider.connection,
    );

    const recentTransaction = indexedTransactions[0];
    assert.equal(indexedTransactions.length, 3);
    assert.equal(
      recentTransaction.amountSpl.div(tokenCtx.decimals).toNumber(),
      amount,
    );
    assert.equal(
      recentTransaction.to.toBase58(),
      recipientSplBalance.toBase58(),
    );
    assert.equal(recentTransaction.type, Action.UNSHIELD);
    assert.equal(recentTransaction.relayerFee.toString(), "500000");
    assert.equal(
      recentTransaction.relayerRecipientSol.toBase58(),
      provider.relayer.accounts.relayerRecipientSol.toBase58(),
    );
  });

  it("(user class) transfer SPL", async () => {
    let amountSpl = 1;
    const token = "USDC";
    const provider = await Provider.init({
      wallet: userKeypair,
      relayer: RELAYER,
    });

    const recipientAccount = new Account({
      poseidon: POSEIDON,
      seed: new Uint8Array(32).fill(9).toString(),
    });

    const recipientAccountFromPubkey = Account.fromPubkey(
      recipientAccount.getPublicKey(),
      POSEIDON,
    );
    // get token from registry
    const tokenCtx = TOKEN_REGISTRY.get(token);

    const user: User = await User.init({ provider });
    const preShieldedBalance = await user.getBalance();
    const previousUtxos = user.utxos;

    await user.transfer({
      amountSpl,
      token,
      recipient: recipientAccountFromPubkey,
    });

    await user.provider.latestMerkleTree();

    let balance = await user.getBalance();

    let tokenBalanceAfter = balance.find(
      (b) => b.tokenAccount.toBase58() === tokenCtx?.mint.toBase58(),
    );
    let tokenBalancePre = preShieldedBalance.find(
      (b) => b.tokenAccount.toBase58() === tokenCtx?.mint.toBase58(),
    );
    // assert that the user's shielded balance has decreased by the amount transferred
    assert.equal(
      tokenBalanceAfter.amount.toNumber(),
      tokenBalancePre.amount.toNumber() -
        amountSpl * tokenCtx?.decimals.toNumber(), // TODO: check that fees go ?
      `shielded balance after ${tokenBalanceAfter.amount} != unshield amount ${
        amountSpl * tokenCtx?.decimals.toNumber()
      }`,
    );
    // assert that the user's sol shielded balance has decreased by fee
    let solBalanceAfter = balance.find(
      (b) => b.tokenAccount.toBase58() === SystemProgram.programId.toBase58(),
    );
    let solBalancePre = preShieldedBalance.find(
      (b) => b.tokenAccount.toBase58() === SystemProgram.programId.toBase58(),
    );
    assert.equal(
      solBalanceAfter.amount.toNumber(),
      solBalancePre.amount.toNumber() - provider.relayer.relayerFee.toNumber(), // FIXME: no fees being charged here apparently
      `shielded sol balance after ${solBalanceAfter.amount} != unshield amount -fee -minimumSplUtxoChanges`,
    );

    assert.notEqual(
      fetchNullifierAccountInfo(user.utxos[0]._nullifier, provider.connection),
      null,
    );

    let commitmentIndex = user.spentUtxos.findIndex(
      (utxo) => utxo._commitment === user.utxos[0]._commitment,
    );

    let commitmentSpent = user.utxos.findIndex(
      (utxo) => utxo._commitment === previousUtxos[0]._commitment,
    );

    assert.equal(user.spentUtxos.length, 3);
    assert.equal(user.utxos.length, 1);
    assert.equal(commitmentIndex, -1);
    assert.equal(commitmentSpent, -1);

    const indexedTransactions = await provider.relayer.getIndexedTransactions(
      provider.provider.connection,
    );
    const recentTransaction = indexedTransactions[0];
    assert.equal(indexedTransactions.length, 4);
    assert.equal(recentTransaction.to.toBase58(), PublicKey.default.toBase58());
    assert.equal(recentTransaction.amountSol.toNumber(), 0);
    assert.equal(recentTransaction.amountSpl.toNumber(), 0);
    assert.equal(
      recentTransaction.from.toBase58(),
      PublicKey.default.toBase58(),
    );
    assert.equal(recentTransaction.type, Action.TRANSFER);
    assert.equal(recentTransaction.relayerFee.toString(), "100000");
    assert.equal(
      recentTransaction.relayerRecipientSol.toBase58(),
      provider.relayer.accounts.relayerRecipientSol.toBase58(),
    );

    // assert recipient utxo
    const userRecipient: User = await User.init({
      provider,
      seed: new Uint8Array(32).fill(9).toString(),
    });

    let { decryptedUtxos } = await userRecipient.getUtxos(false);
    assert.equal(decryptedUtxos.length, 1);
    assert.equal(decryptedUtxos[0].amounts[1].toString(), "100");
  });

  it.skip("(user class) transfer SOL", async () => {
    let amount = 1;
    let token = "SOL";
    let recipientAccountRoot = new Account({poseidon: POSEIDON, seed: bs58.encode(new Uint8Array(32).fill(3))})

    const provider = await Provider.init({
      wallet: userKeypair,
      relayer: RELAYER,
    }); // userKeypair
    // get token from registry
    const tokenCtx = TOKEN_REGISTRY.get(token);

    const user: User = await User.init({ provider });
    const preShieldedBalance = await user.getBalance();

    await user.transfer({
      amountSpl: amount,
      token,
      recipient: Account.fromPubkey(recipientAccountRoot.getPublicKey(), POSEIDON)
    });

    await user.provider.latestMerkleTree();

    let balance = await user.getBalance();

    // assert that the user's sol shielded balance has decreased by fee
    let solBalanceAfter = balance.find(
      (b) => b.tokenAccount.toBase58() === SystemProgram.programId.toString(),
    );
    let solBalancePre = preShieldedBalance.find(
      (b) => b.tokenAccount.toBase58() === SystemProgram.programId.toString(),
    );

    assert.equal(
      solBalanceAfter.amount.toNumber(),
      solBalancePre.amount.toNumber() - 100000 - amount * tokenCtx.decimals.toNumber(),
      `shielded sol balance after ${solBalanceAfter.amount} != ${solBalancePre.amount} ...unshield amount -fee`,
    );
  });

  it.skip("(user class) unshield SOL", async () => {
    let amount = 1;
    let token = "SOL";
    let recipient = new PublicKey(
      "E7jqevikamCMCda8yCsfNawj57FSotUZuref9MLZpWo1",
    );

    const provider = await Provider.init({
      wallet: userKeypair,
      relayer: RELAYER,
    }); // userKeypair

    const user = await User.init({ provider });
    await user.unshield({ amount, token, recipient });
    // TODO: add random amount and amount checks
  });

  it.skip("(user class) shield SOL to recipient", async () => {
    let amount = 15;
    let token = "SOL";
    const provider = await Provider.init({
      wallet: userKeypair,
      relayer: RELAYER,
    }); // userKeypair
    let res = await provider.provider.connection.requestAirdrop(
      userKeypair.publicKey,
      4_000_000_000,
    );
    const recipientAccount = new Account({
      poseidon: POSEIDON,
      seed: new Uint8Array(32).fill(7).toString(),
    });
    await provider.provider.connection.confirmTransaction(res, "confirmed");
    const userSender: User = await User.init({ provider });
    const tokenCtx = TOKEN_REGISTRY.get(token);
    const user: User = await User.init({
      provider,
      seed: new Uint8Array(32).fill(7).toString(),
    });

    const preShieldedBalance = await user.getBalance();
    const preSolBalance = await provider.provider.connection.getBalance(
      userKeypair.publicKey,
    );
    const previousUtxos = user.utxos;

    await userSender.shield({
      publicAmountSol: amount,
      token,
      recipient: recipientAccount,
    });
    // TODO: add random amount and amount checks
    await user.provider.latestMerkleTree();

    let balance = await user.getBalance();
    let solShieldedBalanceAfter = balance.find(
      (b) => b.tokenAccount.toBase58() === tokenCtx?.mint.toBase58(),
    );
    let solShieldedBalancePre = preShieldedBalance.find(
      (b) => b.tokenAccount.toBase58() === tokenCtx?.mint.toBase58(),
    );

    // console.log("solShieldedBalanceAfter", solShieldedBalanceAfter);
    // console.log("solShieldedBalancePre", solShieldedBalancePre);

    // assert that the user's token balance has decreased by the amount shielded
    const postSolBalance = await provider.provider.connection.getBalance(
      userKeypair.publicKey,
    );
    let tempAccountCost = 3502840 - 1255000; //x-y nasty af. underterministic: costs more(y) if shielded SPL before!
    // console.log("postSolBalance", postSolBalance);
    // console.log("preSolBalance", preSolBalance);
    // assert that the user's shielded balance has increased by the amount shielded
    assert.equal(
      solShieldedBalanceAfter.amount.toNumber(),
      solShieldedBalancePre.amount.toNumber() +
        amount * tokenCtx?.decimals.toNumber(),
      `shielded balance after ${
        solShieldedBalanceAfter.amount
      } != shield amount ${amount * tokenCtx?.decimals.toNumber()}`,
    );

    assert.equal(
      postSolBalance,
      preSolBalance - amount * tokenCtx.decimals.toNumber() + tempAccountCost,
      `user token balance after ${postSolBalance} != user token balance before ${preSolBalance} - shield amount ${amount} sol + tempAccountCost! ${tempAccountCost}`,
    );

    assert.notEqual(
      fetchNullifierAccountInfo(user.utxos[0]._nullifier, provider.connection),
      null,
    );

    assert.equal(user.utxos.length, 1);

    const indexedTransactions = await provider.relayer.getIndexedTransactions(
      provider.provider.connection,
    );
    const recentTransaction = indexedTransactions[0];
    assert.equal(
      recentTransaction.amountSol.div(tokenCtx.decimals).toNumber(),
      amount,
    );
    assert.equal(
      recentTransaction.from.toBase58(),
      provider.wallet.publicKey.toBase58(),
    );
    assert.equal(recentTransaction.commitment, user.utxos[0]._commitment);
    assert.equal(recentTransaction.type, Action.SHIELD);
    assert.equal(recentTransaction.relayerFee.toString(), "0");
    assert.equal(
      recentTransaction.relayerRecipientSol.toBase58(),
      PublicKey.default.toBase58(),
    );
  });
});

describe("Test User Errors", () => {
  // Configure the client to use the local cluster.
  process.env.ANCHOR_WALLET = process.env.HOME + "/.config/solana/id.json";
  process.env.ANCHOR_PROVIDER_URL = "http://127.0.0.1:8899";

  const providerAnchor = AnchorProvider.local(
    "http://127.0.0.1:8899",
    confirmConfig,
  );
  anchor.setProvider(providerAnchor);
  console.log("merkleTreeProgram: ", merkleTreeProgramId.toBase58());

  const userKeypair = ADMIN_AUTH_KEYPAIR; //new SolanaKeypair();

  let amount, token, provider, user;
  before("init test setup Merkle tree lookup table etc ", async () => {
    if ((await providerAnchor.connection.getBalance(ADMIN_AUTH_KEY)) === 0) {
      await createTestAccounts(providerAnchor.connection);
      LOOK_UP_TABLE = await initLookUpTableFromFile(providerAnchor);
    }

    POSEIDON = await circomlibjs.buildPoseidonOpt();
    amount = 20;
    token = "USDC";

    provider = await await Provider.init({
      wallet: userKeypair,
      relayer: RELAYER,
    }); // userKeypair
    let res = await provider.provider.connection.requestAirdrop(
      userKeypair.publicKey,
      2_000_000_000,
    );
    await provider.provider.connection.confirmTransaction(res, "confirmed");
    user = await User.init({ provider });
  });
  it("NO_PUBLIC_AMOUNTS_PROVIDED shield", async () => {
    await chai.assert.isRejected(
      user.shield({ token }),
      CreateUtxoErrorCode.NO_PUBLIC_AMOUNTS_PROVIDED,
    );
  });

  it("TOKEN_UNDEFINED shield", async () => {
    await chai.assert.isRejected(
      // @ts-ignore
      user.shield({ publicAmountSpl: amount }),
      UserErrorCode.TOKEN_UNDEFINED,
    );
  });

  it("INVALID_TOKEN shield", async () => {
    await chai.assert.isRejected(
      // @ts-ignore
      user.shield({ publicAmountSpl: amount, token: "SOL" }),
      UserErrorCode.INVALID_TOKEN,
    );
  });

  it("TOKEN_ACCOUNT_DEFINED shield", async () => {
    await chai.assert.isRejected(
      // @ts-ignore
      user.shield({
        publicAmountSol: amount,
        token: "SOL",
        senderTokenAccount: SolanaKeypair.generate().publicKey,
      }),
      UserErrorCode.TOKEN_ACCOUNT_DEFINED,
    );
  });

  it("TOKEN_NOT_FOUND shield", async () => {
    await chai.assert.isRejected(
      // @ts-ignore
      user.shield({ publicAmountSol: amount, token: "SPL" }),
      UserErrorCode.TOKEN_NOT_FOUND,
    );
  });

  it("TOKEN_NOT_FOUND unshield", async () => {
    await chai.assert.isRejected(
      // @ts-ignore
      user.unshield({ amountSol: amount, token: "SPL" }),
      UserErrorCode.TOKEN_NOT_FOUND,
    );
  });

  it("TOKEN_NOT_FOUND transfer", async () => {
    await chai.assert.isRejected(
      // @ts-ignore
      user.unshield({ amountSol: amount, token: "SPL" }),
      UserErrorCode.TOKEN_NOT_FOUND,
    );
  });

  it("NO_PUBLIC_AMOUNTS_PROVIDED unshield", async () => {
    await chai.assert.isRejected(
      user.unshield({ token }),
      CreateUtxoErrorCode.NO_PUBLIC_AMOUNTS_PROVIDED,
    );
  });

  it("TOKEN_NOT_FOUND unshield", async () => {
    await chai.assert.isRejected(
      // @ts-ignore
      user.unshield({}),
      UserErrorCode.TOKEN_NOT_FOUND,
    );
  });

  it("SOL_RECIPIENT_UNDEFINED unshield", async () => {
    await chai.assert.isRejected(
      // @ts-ignore
      user.unshield({ token: "SOL", publicAmountSol: new BN(1) }),
      TransactionErrorCode.SOL_RECIPIENT_UNDEFINED,
    );

    await chai.assert.isRejected(
      // @ts-ignore
      user.unshield({
        token,
        publicAmountSol: new BN(1),
        publicAmountSpl: new BN(1),
        recipientSpl: SolanaKeypair.generate().publicKey,
      }),
      TransactionErrorCode.SOL_RECIPIENT_UNDEFINED,
    );
  });

  it("SPL_RECIPIENT_UNDEFINED unshield", async () => {
    await chai.assert.isRejected(
      // @ts-ignore
      user.unshield({ token, publicAmountSpl: new BN(1) }),
      TransactionErrorCode.SPL_RECIPIENT_UNDEFINED,
    );
  });

  it("TOKEN_NOT_FOUND shield", async () => {
    await chai.assert.isRejected(
      // @ts-ignore
      user.shield({ publicAmountSol: SolanaKeypair.generate().publicKey }),
      UserErrorCode.TOKEN_NOT_FOUND,
    );
  });

  it("TOKEN_NOT_FOUND transfer", async () => {
    await chai.assert.isRejected(
      // @ts-ignore
      user.transfer({
        recipient: new Account({ poseidon: POSEIDON }),
        amountSol: new BN(1),
      }),
      UserErrorCode.TOKEN_NOT_FOUND,
    );
  });

  it("SHIELDED_RECIPIENT_UNDEFINED transfer", async () => {
    await chai.assert.isRejected(
      // @ts-ignore
      user.transfer({}),
      UserErrorCode.SHIELDED_RECIPIENT_UNDEFINED,
    );
  });

  it("NO_AMOUNTS_PROVIDED transfer", async () => {
    await chai.assert.isRejected(
      // @ts-ignore
      user.transfer({ recipient: new Account({ poseidon: POSEIDON }) }),
      UserErrorCode.NO_AMOUNTS_PROVIDED,
    );
  });
});
