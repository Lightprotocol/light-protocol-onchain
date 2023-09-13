import express from "express";
import { DB_VERSION, port } from "./config";
import { addCorsHeaders } from "./middleware";
import bodyParser from "body-parser";
import {
  getIndexedTransactions,
  buildMerkleTree,
  updateMerkleTree,
  handleRelayRequest,
  runIndexer,
  getLookUpTable,
} from "./services";
import { getTransactions } from "./db/redis";
import { createTestAccounts } from "@lightprotocol/zk.js";
import { getAnchorProvider } from "./utils/provider";
import { setupRelayerLookUpTable } from "./setup";
require("dotenv").config();

const app = express();

app.use(addCorsHeaders);
app.use(bodyParser.json());

app.get("/", async (_req: any, res: any) => {
  try {
    return res.status(200).json({ status: "gm." });
  } catch (e) {
    console.log(e);
    return res.status(500).json({ status: "error", message: e.message });
  }
});

app.post("/updatemerkletree", updateMerkleTree);

app.get("/getBuiltMerkletree", buildMerkleTree);

app.get("/lookuptable", getLookUpTable);

app.post("/relayTransaction", handleRelayRequest);

app.get("/indexedTransactions", getIndexedTransactions);

app.listen(port, async () => {
  const anchorProvider = await getAnchorProvider();

  await setupRelayerLookUpTable(anchorProvider);
  console.log("Relayer lookuptable set up!");
  if (process.env.TEST_ENVIRONMENT) {
    // TODO: separate! tesnet too
    // if (process.env.ENVIRONMENT !== Environment.PROD) await fundRelayer();
    await createTestAccounts(anchorProvider.connection);
    console.log("Test environment setup completed!");
    // TODO: temporary!
    let { job } = await getTransactions(DB_VERSION);
    await job.updateData({ transactions: [] });
  }

  runIndexer();

  console.log(`Webserver started on port ${port}`);
  console.log("rpc:", process.env.RPC_URL);
});
