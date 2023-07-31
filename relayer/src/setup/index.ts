import {
  createTestAccounts,
  initLookUpTable,
  useWallet,
} from "@lightprotocol/zk.js";
import { getAnchorProvider, getKeyPairFromEnv } from "../utils/provider";
import { PublicKey } from "@solana/web3.js";
import { readFileSync, writeFile, writeFileSync } from "fs";

export const testSetup = async () => {
  const providerAnchor = await getAnchorProvider();
  // TODO: use updated -- buildscript -> add relayer tests
  await createTestAccounts(providerAnchor.connection);

  let lookUpTable;
  const path = "lookUpTable.txt";
  try {
    let lookUpTableRead = new PublicKey(readFileSync(path, "utf8"));
    let lookUpTableInfoInit = await providerAnchor.connection.getAccountInfo(
      lookUpTableRead,
    );
    if (lookUpTableInfoInit) {
      lookUpTable = lookUpTableRead;
    }
  } catch (e) {
    console.log(".txt not found", e);
  }
  if (!lookUpTable) {
    lookUpTable = await initLookUpTable(
      useWallet(getKeyPairFromEnv("KEY_PAIR")),
      providerAnchor,
    );

    writeFileSync(path, lookUpTable.toString(), "utf8");
  }
};
