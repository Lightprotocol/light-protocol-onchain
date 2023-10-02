import {
  FEE_ASSET,
  Account,
  Provider as LightProvider,
  MINT,
  Utxo,
  Transaction,
  Action,
  TransactionParameters,
  IDL_VERIFIER_PROGRAM_ZERO,
} from "../index";
import * as anchor from "@coral-xyz/anchor";
import { Keypair as SolanaKeypair } from "@solana/web3.js";
import { Idl } from "@coral-xyz/anchor";
import { bs58 } from "@coral-xyz/anchor/dist/cjs/utils/bytes";
const circomlibjs = require("circomlibjs");

export async function functionalCircuitTest(
  app: boolean = false,
  verifierIdl: Idl,
) {
  let lightProvider = await LightProvider.loadMock();

  const poseidon = await circomlibjs.buildPoseidonOpt();
  let seed32 = bs58.encode(new Uint8Array(32).fill(1));
  let account = new Account({ poseidon: poseidon, seed: seed32 });
  let depositAmount = 20_000;
  let depositFeeAmount = 10_000;
  let deposit_utxo1 = new Utxo({
    poseidon: poseidon,
    assets: [FEE_ASSET, MINT],
    amounts: [new anchor.BN(depositFeeAmount), new anchor.BN(depositAmount)],
    publicKey: account.pubkey,
    assetLookupTable: lightProvider.lookUpTables.assetLookupTable,
    verifierProgramLookupTable:
      lightProvider.lookUpTables.verifierProgramLookupTable,
  });
  let mockPubkey = SolanaKeypair.generate().publicKey;

  let txParams = new TransactionParameters({
    outputUtxos: [deposit_utxo1],
    eventMerkleTreePubkey: mockPubkey,
    transactionMerkleTreePubkey: mockPubkey,
    senderSpl: mockPubkey,
    senderSol: lightProvider.wallet!.publicKey,
    action: Action.SHIELD,
    poseidon,
    verifierIdl: verifierIdl,
    account,
  });

  let tx;
  const { rootIndex, remainingAccounts } = await lightProvider.getRootIndex();
  // successful proof generation
  if (app) {
    tx = new Transaction({
      rootIndex,
      nextTransactionMerkleTree: remainingAccounts.nextTransactionMerkleTree,
      solMerkleTree: lightProvider.solMerkleTree!,
      params: txParams,
      appParams: {
        mock: "123",
        // just a placeholder the test does not compute an app proof
        verifierIdl: IDL_VERIFIER_PROGRAM_ZERO,
        path: "./build-circuits",
      },
    });
  } else {
    tx = new Transaction({
      rootIndex,
      nextTransactionMerkleTree: remainingAccounts.nextTransactionMerkleTree,
      solMerkleTree: lightProvider.solMerkleTree!,
      params: txParams,
    });
  }
  await tx.compile(lightProvider.poseidon, account);

  await tx.getProof(account);
  // unsuccessful proof generation
  let x = true;

  try {
    tx.proofInput.inIndices[0][1][1] = "1";
    // TODO: investigate why this does not kill the proof
    tx.proofInput.inIndices[0][1][0] = "1";
    await tx.getProof(account);
    x = false;
  } catch (error) {
    if (!error.toString().includes("CheckIndices_3 line: 34")) {
      throw new Error(
        "Expected error to be CheckIndices_3, but it was " + error.toString(),
      );
    }
  }
  if (!x) {
    throw new Error("Expected value to be true, but it was false.");
  }
}
