import {
  AddressLookupTableAccount,
  BlockheightBasedTransactionConfirmationStrategy,
  ComputeBudgetProgram,
  ConfirmOptions,
  Connection,
  Keypair,
  PublicKey,
  TransactionConfirmationStrategy,
  TransactionInstruction,
  TransactionMessage,
  TransactionSignature,
  VersionedTransaction,
} from "@solana/web3.js";

import { Wallet } from "../wallet";
import { confirmConfig } from "../constants";
export const sendVersionedTransaction = async (
  ix: TransactionInstruction,
  connection: Connection,
  lookUpTable: PublicKey,
  payer: Wallet,
) => {
  const recentBlockhash = (await connection.getLatestBlockhash(confirmConfig))
    .blockhash;
  const ixSigner = ix.keys
    .map((key) => {
      if (key.isSigner == true) return key.pubkey;
    })[0]
    ?.toBase58();
  if (payer.publicKey.toBase58() != ixSigner) {
    throw new Error(
      ` payer pubkey is not equal instruction signer ${payer.publicKey.toBase58()} != ${ixSigner} (only one signer supported)`,
    );
  }
  const txMsg = new TransactionMessage({
    payerKey: payer.publicKey,
    instructions: [
      ComputeBudgetProgram.setComputeUnitLimit({ units: 1_400_000 }),
      ix,
    ],
    recentBlockhash: recentBlockhash,
  });

  const lookupTableAccount = await connection.getAccountInfo(
    lookUpTable,
    "confirmed",
  );

  const unpackedLookupTableAccount = AddressLookupTableAccount.deserialize(
    lookupTableAccount!.data,
  );

  const compiledTx = txMsg.compileToV0Message([
    {
      state: unpackedLookupTableAccount,
      key: lookUpTable,
      isActive: () => {
        return true;
      },
    },
  ]);
  if (compiledTx.addressTableLookups[0]) {
    compiledTx.addressTableLookups[0].accountKey = lookUpTable;
  }

  var tx = new VersionedTransaction(compiledTx);
  let retries = 3;
  let res;
  while (retries > 0) {
    tx = await payer.signTransaction(tx);
    try {
      res = await connection.sendTransaction(tx, confirmConfig);
      retries = 0;
    } catch (e: any) {
      retries--;
      if (retries == 0 || e.logs !== undefined) {
        console.log(e);
        return e;
      }
    }
  }
  return res;
};

export type SendVersionedTransactionsResult = {
  signatures?: TransactionSignature[];
  error?: any;
};

export async function sendVersionedTransactions(
  instructions: any[],
  connection: Connection,
  lookUpTable: PublicKey,
  payer: Wallet,
): Promise<SendVersionedTransactionsResult> {
  let signature;
  try {
    var signatures: TransactionSignature[] = [];
    for (var instruction of instructions) {
      signature = await sendVersionedTransaction(
        instruction,
        connection,
        lookUpTable,
        payer,
      );
      signatures.push(signature);
      const latestBlockHash = await connection.getLatestBlockhash("confirmed");
      let strategy: TransactionConfirmationStrategy = {
        signature: signature.toString(),
        lastValidBlockHeight: latestBlockHash.lastValidBlockHeight,
        blockhash: latestBlockHash.blockhash,
      };
      await connection.confirmTransaction(strategy);
    }
    return { signatures };
  } catch (error) {
    return { error: signature };
  }
}
