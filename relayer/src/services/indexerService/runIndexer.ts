import { DB_VERSION, SECONDS } from "../../config";
import { indexTransactions } from "./indexer";
import { Connection } from "@solana/web3.js";
import { getTransactions } from "../../db/redis";
import { sleep } from "@lightprotocol/zk.js";
import {
  EnvironmentVariableError,
  EnvironmentVariableErrorCode,
} from "../../errors";

/// TODO: once we add webhooks for forward indexing, we can turn runIndexer() into fillBackward().
export async function runIndexer(rounds: number = 0) {
  console.log("runIndexer initializing...");
  await getTransactions(DB_VERSION);
  console.log("initialized");
  var fillBackward = true;
  let laps = -1;
  while (laps < rounds) {
    if (fillBackward) await sleep(3 * SECONDS);
    else await sleep(5 * SECONDS);
    const { job } = await getTransactions(DB_VERSION);
    const url = process.env.RPC_URL;
    if (!url)
      throw new EnvironmentVariableError(
        EnvironmentVariableErrorCode.VARIABLE_NOT_SET,
        "runIndexer",
        "RPC_URL",
      );
    const connection = new Connection(url, "confirmed");

    if (job) {
      console.log(
        `transactions indexed in db v${DB_VERSION}: ${job.data.transactions.length}`,
      );
    }
    let { continueBackwardFill } = await indexTransactions({
      job,
      connection,
      fillBackward,
    });
    fillBackward = continueBackwardFill;
    if (rounds !== 0) {
      laps++;
    }
  }
}
