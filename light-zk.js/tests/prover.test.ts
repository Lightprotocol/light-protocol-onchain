import { assert, expect } from "chai";
import { Prover } from "../src/transaction/prover";
import { Idl } from "@coral-xyz/anchor";

let circomlibjs = require("circomlibjs");
import { Keypair as SolanaKeypair } from "@solana/web3.js";
import * as anchor from "@coral-xyz/anchor";

import { it } from "mocha";
const chai = require("chai");
const chaiAsPromised = require("chai-as-promised");

// Load chai-as-promised support
chai.use(chaiAsPromised);
import {
  FEE_ASSET,
  Provider as LightProvider,
  MINT,
  Transaction,
  TransactionParameters,
  Action,
  Relayer,
  Utxo,
  Account,
  MerkleTree,
  IDL_VERIFIER_PROGRAM_ZERO,
} from "../src";

process.env.ANCHOR_PROVIDER_URL = "http://127.0.0.1:8899";
process.env.ANCHOR_WALLET = process.env.HOME + "/.config/solana/id.json";

describe("Test Prover Functional", () => {
  let seed32 = new Uint8Array(32).fill(1).toString();
  let depositAmount = 20_000;
  let depositFeeAmount = 10_000;

  let mockPubkey = SolanaKeypair.generate().publicKey;
  let mockPubkey2 = SolanaKeypair.generate().publicKey;
  let mockPubkey3 = SolanaKeypair.generate().publicKey;
  let poseidon: any,
    lightProvider: LightProvider,
    deposit_utxo1,
    relayer,
    keypair: Account,
    paramsDeposit: TransactionParameters,
    paramsWithdrawal;
  before(async () => {
    poseidon = await circomlibjs.buildPoseidonOpt();
    // TODO: make fee mandatory
    relayer = new Relayer(mockPubkey3, mockPubkey, new anchor.BN(5000));
    keypair = new Account({ poseidon: poseidon, seed: seed32 });
    lightProvider = await LightProvider.loadMock();
    deposit_utxo1 = new Utxo({
      poseidon: poseidon,
      assets: [FEE_ASSET, MINT],
      amounts: [new anchor.BN(depositFeeAmount), new anchor.BN(depositAmount)],
      account: keypair,
      blinding: new anchor.BN(new Array(31).fill(1)),
      assetLookupTable: lightProvider.lookUpTables.assetLookupTable,
      verifierProgramLookupTable:
        lightProvider.lookUpTables.verifierProgramLookupTable,
    });
    paramsDeposit = new TransactionParameters({
      outputUtxos: [deposit_utxo1],
      eventMerkleTreePubkey: mockPubkey2,
      transactionMerkleTreePubkey: mockPubkey2,
      poseidon,
      senderSpl: mockPubkey,
      senderSol: lightProvider.wallet?.publicKey,
      action: Action.SHIELD,
      verifierIdl: IDL_VERIFIER_PROGRAM_ZERO,
    });
    lightProvider.solMerkleTree!.merkleTree = new MerkleTree(18, poseidon, [
      deposit_utxo1.getCommitment(poseidon),
    ]);

    assert.equal(
      lightProvider.solMerkleTree?.merkleTree.indexOf(
        deposit_utxo1.getCommitment(poseidon),
      ),
      0,
    );
    paramsWithdrawal = new TransactionParameters({
      inputUtxos: [deposit_utxo1],
      eventMerkleTreePubkey: mockPubkey2,
      transactionMerkleTreePubkey: mockPubkey2,
      poseidon,
      recipientSpl: mockPubkey,
      recipientSol: lightProvider.wallet?.publicKey,
      action: Action.UNSHIELD,
      relayer,
      verifierIdl: IDL_VERIFIER_PROGRAM_ZERO,
    });
  });

  it("prover functional test1", async () => {
    let tx = new Transaction({
      provider: lightProvider,
      params: paramsDeposit,
    });
    await tx.compile();
    await tx.getProof();

    await tx.getRootIndex();
    tx.getPdaAddresses();
  });

  it("prover functional compileAndProve test", async () => {
    let tx = new Transaction({
      provider: lightProvider,
      params: paramsDeposit,
    });
    await tx.compileAndProve();
  });

  it("test Prover class in transaction", async () => {
    let tx = new Transaction({
      provider: lightProvider,
      params: paramsDeposit,
    });

    await tx.compile();
    const prover = new Prover(
      tx.params.verifierIdl as Idl,
      tx.firstPath as string,
    );
    await prover.addProofInputs(tx.proofInput);
    await prover.fullProve();
    await tx.getProof();

    const publicInputsBytes = prover.parseToBytesArray(prover.publicInputs);
    const { unstringifyBigInts, leInt2Buff } = require("ffjavascript").utils;
    const publicInputsJson = JSON.stringify(prover.publicInputs, null, 1);

    let publicInputsBytesJson = JSON.parse(publicInputsJson.toString());
    let publicInputsBytesVerifier = new Array<Array<number>>();
    for (let i in publicInputsBytesJson) {
      let ref: Array<number> = Array.from([
        ...leInt2Buff(unstringifyBigInts(publicInputsBytesJson[i]), 32),
      ]).reverse();
      publicInputsBytesVerifier.push(ref);
    }

    expect(publicInputsBytes).to.deep.equal(publicInputsBytesVerifier);
  });

  it("prover functional test2", async () => {
    const deposit_utxo1 = new Utxo({
      poseidon: poseidon,
      assets: [FEE_ASSET, MINT],
      amounts: [new anchor.BN(depositFeeAmount), new anchor.BN(depositAmount)],
      account: keypair,
      blinding: new anchor.BN(new Array(31).fill(1)),
      assetLookupTable: lightProvider.lookUpTables.assetLookupTable,
      verifierProgramLookupTable:
        lightProvider.lookUpTables.verifierProgramLookupTable,
    });
    const zeroUtxo1 = new Utxo({
      poseidon: poseidon,
      account: keypair,
      blinding: new anchor.BN(new Array(31).fill(1)),
      assetLookupTable: lightProvider.lookUpTables.assetLookupTable,
      verifierProgramLookupTable:
        lightProvider.lookUpTables.verifierProgramLookupTable,
    });
    const zeroUtxo2 = new Utxo({
      poseidon: poseidon,
      account: keypair,
      blinding: new anchor.BN(new Array(31).fill(2)),
      assetLookupTable: lightProvider.lookUpTables.assetLookupTable,
      verifierProgramLookupTable:
        lightProvider.lookUpTables.verifierProgramLookupTable,
    });
    const paramsDeposit = new TransactionParameters({
      outputUtxos: [deposit_utxo1, zeroUtxo1],
      inputUtxos: [zeroUtxo1, zeroUtxo2],
      eventMerkleTreePubkey: mockPubkey2,
      transactionMerkleTreePubkey: mockPubkey2,
      poseidon,
      senderSpl: mockPubkey,
      senderSol: lightProvider.wallet?.publicKey,
      action: Action.SHIELD,
      verifierIdl: IDL_VERIFIER_PROGRAM_ZERO,
      encryptedUtxos: new Uint8Array(256).fill(2),
    });
    let tx = new Transaction({
      provider: lightProvider,
      params: paramsDeposit,
    });
    await tx.compile();
    const prover = new Prover(tx.params.verifierIdl, tx.firstPath);
    await prover.addProofInputs(tx.proofInput);
    await prover.fullProve();

    await tx.getProof();

    // assert compliance of constant publicInputsBytes
    const hardcodedPublicInputs = {
      root: [
        1, 71, 64, 152, 213, 69, 238, 111, 106, 174, 120, 195, 68, 9, 81, 21,
        57, 227, 243, 231, 251, 182, 3, 222, 79, 89, 20, 111, 194, 140, 137, 35,
      ],
      publicAmountSpl: [
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 78, 32,
      ],
      txIntegrityHash: [
        26, 107, 16, 0, 121, 210, 135, 105, 133, 216, 225, 195, 29, 17, 137,
        235, 197, 31, 198, 221, 54, 120, 128, 37, 138, 104, 186, 202, 238, 12,
        113, 209,
      ],
      publicAmountSol: [
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 39, 16,
      ],
      publicMintPubkey: [
        0, 24, 59, 207, 17, 191, 51, 84, 25, 96, 177, 164, 233, 142, 128, 208,
        115, 82, 0, 223, 237, 121, 0, 231, 241, 213, 140, 224, 58, 185, 152,
        253,
      ],
      inputNullifier: [
        [
          24, 246, 238, 176, 229, 41, 194, 92, 119, 242, 37, 255, 251, 141, 79,
          103, 163, 82, 170, 245, 43, 254, 173, 155, 218, 16, 161, 4, 181, 103,
          231, 25,
        ],
        [
          10, 200, 116, 173, 79, 92, 131, 56, 52, 94, 25, 249, 88, 77, 52, 215,
          145, 78, 131, 112, 85, 61, 183, 167, 124, 59, 233, 144, 36, 128, 60,
          243,
        ],
      ],
      outputCommitment: [
        [
          5, 59, 246, 158, 197, 149, 171, 83, 1, 169, 89, 112, 145, 137, 194,
          212, 10, 206, 172, 194, 240, 70, 141, 203, 248, 111, 9, 239, 114, 31,
          172, 53,
        ],
        [
          45, 33, 14, 89, 191, 213, 234, 199, 195, 91, 43, 8, 143, 46, 130, 238,
          53, 136, 229, 186, 73, 125, 201, 35, 226, 204, 84, 135, 18, 189, 41,
          238,
        ],
      ],
    };
    expect(tx.transactionInputs.publicInputs).to.deep.equal(
      hardcodedPublicInputs,
    );
    await tx.getRootIndex();
    tx.getPdaAddresses();
  });
});
