import { BN } from "@coral-xyz/anchor";
import {
  confirmConfig,
  merkleTreeProgram,
  merkleTreeProgramId,
  MESSAGE_MERKLE_TREE_KEY,
  TRANSACTION_MERKLE_TREE_KEY,
} from "./constants";
import { Connection, PublicKey, SystemProgram } from "@solana/web3.js";
import { MerkleTreeConfig, SolMerkleTree } from "./merkleTree";
import { MINT } from "./test-utils/constants_system_verifier";
import * as anchor from "@coral-xyz/anchor";
import { Utxo } from "utxo";
import { MetaError, UtilsError, UtilsErrorCode } from "./errors";
import { TokenUtxoBalance } from "wallet";
import { TokenData } from "types";
const { keccak_256 } = require("@noble/hashes/sha3");

export function hashAndTruncateToCircuit(data: Uint8Array) {
  return new BN(
    keccak_256
      .create({ dkLen: 32 })
      .update(Buffer.from(data))
      .digest()
      .slice(1, 32),
    undefined,
    "be",
  );
}

// TODO: add pooltype
export async function getAssetLookUpId({
  connection,
  asset,
}: {
  asset: PublicKey;
  connection: Connection;
  // poolType?: Uint8Array
}): Promise<any> {
  let poolType = new Array(32).fill(0);
  let mtConf = new MerkleTreeConfig({
    connection,
    messageMerkleTreePubkey: MESSAGE_MERKLE_TREE_KEY,
    transactionMerkleTreePubkey: TRANSACTION_MERKLE_TREE_KEY,
  });
  let pubkey = await mtConf.getSplPoolPda(asset, poolType);

  let registeredAssets =
    await mtConf.merkleTreeProgram.account.registeredAssetPool.fetch(
      pubkey.pda,
    );
  return registeredAssets.index;
}

// TODO: fetch from chain
// TODO: separate testing variables from prod env
export const assetLookupTable: string[] = [
  SystemProgram.programId.toBase58(),
  MINT.toBase58(),
];

export const verifierLookupTable: string[] = [
  SystemProgram.programId.toBase58(),
  "Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS",
];

export function getAssetIndex(assetPubkey: PublicKey): BN {
  return new BN(assetLookupTable.indexOf(assetPubkey.toBase58()));
}

export function fetchAssetByIdLookUp(assetIndex: BN): PublicKey {
  return new PublicKey(assetLookupTable[assetIndex.toNumber()]);
}

export function fetchVerifierByIdLookUp(index: BN): PublicKey {
  return new PublicKey(verifierLookupTable[index.toNumber()]);
}

export const arrToStr = (uint8arr: Uint8Array) =>
  "LPx" + Buffer.from(uint8arr.buffer).toString("hex");

export const strToArr = (str: string) =>
  new Uint8Array(Buffer.from(str.slice(3), "hex"));

export function decimalConversion({
  tokenCtx,
  skipDecimalConversions,
  publicAmountSpl,
  publicAmountSol,
  minimumLamports,
  minimumLamportsAmount,
}: {
  tokenCtx: TokenData;
  skipDecimalConversions?: boolean;
  publicAmountSpl?: BN | string | number;
  publicAmountSol?: BN | string | number;
  minimumLamports?: boolean;
  minimumLamportsAmount?: BN;
}) {
  if (!skipDecimalConversions) {
    publicAmountSpl = publicAmountSpl
      ? convertAndComputeDecimals(publicAmountSpl, tokenCtx.decimals)
      : undefined;
    // If SOL amount is not provided, the default value is either minimum amount (if defined) or 0.
    publicAmountSol = publicAmountSol
      ? convertAndComputeDecimals(publicAmountSol, new BN(1e9))
      : minimumLamports
      ? minimumLamportsAmount
      : new BN(0);
  } else {
    publicAmountSpl = publicAmountSpl
      ? new BN(publicAmountSpl.toString())
      : undefined;
    publicAmountSol = publicAmountSol
      ? new BN(publicAmountSol?.toString())
      : new BN(0);
  }
  return { publicAmountSpl, publicAmountSol };
}
export const convertAndComputeDecimals = (
  amount: BN | string | number,
  decimals: BN,
) => {
  if (typeof amount === "number" && amount < 0) {
    throw new Error("Negative amounts are not allowed.");
  }

  if (typeof amount === "string" && amount.startsWith("-")) {
    throw new Error("Negative amounts are not allowed.");
  }
  if (decimals.lt(new BN(0))) {
    throw new Error("Negative decimals are not allowed.");
  }

  let amountStr = amount.toString();

  if (amountStr.includes(".")) {
    let [whole, fractional] = amountStr.split(".");
    if (fractional.length >= decimals.toString().length) {
      throw new Error("The amount has more decimal places than allowed.");
    }
    while (fractional.length < decimals.toString().length - 1) {
      fractional += "0"; // Add trailing zeros to match the decimals count
    }
    const res = whole + fractional;
    return new BN(res);
  }

  const bnAmount = new BN(amountStr);
  if (bnAmount.lt(new BN("0")))
    throw new Error("Negative amounts are not allowed.");
  if (decimals.toString() === "0") decimals = new BN(1);
  return bnAmount.mul(decimals);
};

export const getUpdatedSpentUtxos = (
  tokenBalances: Map<string, TokenUtxoBalance>,
): Utxo[] => {
  return Array.from(tokenBalances.values())
    .map((value) => Array.from(value.spentUtxos.values()))
    .flat();
};

export const fetchNullifierAccountInfo = async (
  nullifier: string,
  connection: Connection,
) => {
  const nullifierPubkey = PublicKey.findProgramAddressSync(
    [
      Buffer.from(new anchor.BN(nullifier.toString()).toArray()),
      anchor.utils.bytes.utf8.encode("nf"),
    ],
    merkleTreeProgramId,
  )[0];
  var retries = 2;
  while (retries > 0) {
    const res = await connection.getAccountInfo(nullifierPubkey, "processed");
    if (res) return res;
    retries--;
  }
  return connection.getAccountInfo(nullifierPubkey, "processed");
};

// use
export const fetchQueuedLeavesAccountInfo = async (
  leftLeaf: Uint8Array,
  connection: Connection,
) => {
  const queuedLeavesPubkey = PublicKey.findProgramAddressSync(
    [leftLeaf, anchor.utils.bytes.utf8.encode("leaves")],
    merkleTreeProgramId,
  )[0];
  return connection.getAccountInfo(queuedLeavesPubkey, "confirmed");
};

export const sleep = (ms: number) => {
  return new Promise((resolve) => setTimeout(resolve, ms));
};

// export var logger = (function () {
//   var oldConsoleLog: any = null;
//   var pub = {};

//   //@ts-ignore
//   pub.enableLogger = function enableLogger() {
//     if (oldConsoleLog == null) return;

//     console.log = oldConsoleLog;
//   };

//   //@ts-ignore
//   pub.disableLogger = function disableLogger() {
//     oldConsoleLog = console.log;
//     window["console"]["log"] = function () {};
//   };

//   return pub;
// })();

export type KeyValue = {
  [key: string]: any;
};
/**
 * @description Creates an object of a type defined in accounts[accountName],
 * @description all properties need to be part of obj, if a property is missing an error is thrown.
 * @description The accounts array is part of an anchor idl.
 * @param obj Object properties are picked from.
 * @param accounts Idl accounts array from which accountName is selected.
 * @param accountName Defines which account in accounts to use as type for the output object.
 * @returns
 */
export function createAccountObject<T extends KeyValue>(
  obj: T,
  accounts: any[],
  accountName: string,
): Partial<KeyValue> {
  const account = accounts.find((account) => account.name === accountName);

  if (!account) {
    throw new UtilsError(
      UtilsErrorCode.ACCOUNT_NAME_UNDEFINED_IN_IDL,
      "pickFieldsFromObject",
      `${accountName} does not exist in idl`,
    );
  }

  const fieldNames = account.type.fields.map(
    (field: { name: string }) => field.name,
  );

  let accountObject: Partial<T> = {};
  fieldNames.forEach((fieldName: keyof T) => {
    accountObject[fieldName] = obj[fieldName];
    if (!accountObject[fieldName])
      throw new UtilsError(
        UtilsErrorCode.PROPERTY_UNDEFINED,
        "pickFieldsFromObject",
        `Property ${fieldName.toString()} undefined`,
      );
  });
  return accountObject;
}

export function firstLetterToLower(input: string): string {
  if (!input) return input;
  return input.charAt(0).toLowerCase() + input.slice(1);
}

export function firstLetterToUpper(input: string): string {
  if (!input) return input;
  return input.charAt(0).toUpperCase() + input.slice(1);
}

/**
 * This function checks if an account in the provided idk object exists with a name
 * ending with 'PublicInputs' and contains a field named 'publicAppVerifier'.
 *
 * @param {Idl} idl - The IDL object to check.
 * @returns {boolean} - Returns true if such an account exists, false otherwise.
 */
export function isProgramVerifier(idl: anchor.Idl): boolean {
  if (!idl.accounts) throw new Error("Idl does not contain accounts");
  return idl.accounts.some(
    (account) =>
      account.name.endsWith("PublicInputs") &&
      account.type.fields.some((field) => field.name === "publicAppVerifier"),
  );
}
