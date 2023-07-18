import { Job } from "bullmq";
import { Connection, TransactionResponse } from "@solana/web3.js";
import { IndexedTransaction } from "@lightprotocol/zk.js";
import { searchBackward, searchForward } from "./search";

// Unused until we move to devnet for webhooks
async function indexStreamedTransactions({
  connection,
  newRawTransactions,
  job,
  token,
}: {
  connection: Connection;
  newRawTransactions: TransactionResponse[];
  job: Job;
  token: any;
}) {
  const newParsedTransactions: IndexedTransaction[] = [];
  //   await Promise.all(
  //     newRawTransactions.map((tx: TransactionResponse) =>
  //       parse({
  //         tx,
  //         transactions: newParsedTransactions,
  //         connection,
  //         token,
  //       }),
  //     ),
  //   );

  let mergedTransactions = mergeAndSortTransactions(job.data.transactions, [
    newParsedTransactions,
  ]);

  await job.updateData({
    transactions: mergedTransactions,
    lastFetched: Date.now(),
  });
}

function mergeAndSortTransactions(
  dbTransactions: IndexedTransaction[],
  newTransactions: IndexedTransaction[][],
) {
  let mergedTransactions: IndexedTransaction[] = dbTransactions.concat(
    ...newTransactions,
  );
  let dedupedTransactions = mergedTransactions.reduce(
    (acc: IndexedTransaction[], cur: IndexedTransaction) => {
      if (cur && !acc.find((item) => item.signature === cur.signature)) {
        acc.push(cur);
      }
      return acc;
    },
    [],
  );
  dedupedTransactions.sort(
    (a: IndexedTransaction, b: IndexedTransaction) => b.blockTime - a.blockTime,
  );
  return dedupedTransactions;
}

export async function indexTransactions({
  job,
  RPC_connection,
  initialSync,
}: {
  job: Job;
  RPC_connection: Connection;
  initialSync: boolean;
}) {
  try {
    const olderTransactions: IndexedTransaction[] = await searchBackward(
      job,
      RPC_connection,
    );
    if (olderTransactions.length === 0) initialSync = false;
    else initialSync = true;

    const newerTransactions: IndexedTransaction[] = await searchForward(
      job,
      RPC_connection,
    );

    let dedupedTransactions: IndexedTransaction[] = mergeAndSortTransactions(
      job.data.transactions,
      [olderTransactions, newerTransactions],
    );
    console.log(
      `new total: ${dedupedTransactions.length} transactions old: ${job.data.transactions.length}, older: ${olderTransactions.length}, newer: ${newerTransactions.length}`,
    );

    await job.updateData({
      transactions: dedupedTransactions,
      lastFetched: Date.now(),
    });
  } catch (e) {
    console.log("restarting indexer -- crash reason:", e);
  }
}
