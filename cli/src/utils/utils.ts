import * as fs from "fs";
import * as path from "path";
import * as os from "os";
import * as anchor from "@coral-xyz/anchor";
import * as solana from "@solana/web3.js";
const spinner = require("cli-spinners");
import { BN } from "@coral-xyz/anchor";
import {
  confirmConfig,
  ConfirmOptions,
  MerkleTreeConfig,
  Provider,
  Relayer,
  RELAYER_FEE,
  TestRelayer,
  TOKEN_ACCOUNT_FEE,
  User,
} from "@lightprotocol/zk.js";
import { bs58 } from "@coral-xyz/anchor/dist/cjs/utils/bytes";
import { CONFIG_FILE_NAME, CONFIG_PATH, DEFAULT_CONFIG } from "../psp-utils";
import { file } from "@oclif/core/lib/flags";

require("dotenv").config();

let provider: Provider;
let relayer: Relayer;

export const createNewWallet = () => {
  const keypair: solana.Keypair = solana.Keypair.generate();
  const secretKey: solana.Ed25519SecretKey = keypair.secretKey;
  try {
    setSecretKey(JSON.stringify(Array.from(secretKey)));
    return keypair;
  } catch (error) {
    throw new Error(`error writing secret.txt: ${error}`);
  }
};

export const getWalletConfig = async (
  connection: solana.Connection
): Promise<MerkleTreeConfig> => {
  try {
    let merkleTreeConfig = new MerkleTreeConfig({
      payer: getPayer(),
      connection,
    });

    await merkleTreeConfig.getMerkleTreeAuthorityPda();

    return merkleTreeConfig;
  } catch (error) {
    console.log({ error });
    throw error;
  }
};

export const readWalletFromFile = () => {
  try {
    const secretKey = bs58.decode(getSecretKey());
    const keypair = solana.Keypair.fromSecretKey(new Uint8Array(secretKey));

    return keypair;
  } catch (error) {
    throw new Error("Secret key not found or corrupted!");
  }
};

export const setAnchorProvider = async (): Promise<anchor.AnchorProvider> => {
  process.env.ANCHOR_WALLET = process.env.HOME + "/.config/solana/id.json";
  process.env.ANCHOR_PROVIDER_URL = getRpcUrl();
  const connection = new solana.Connection(getRpcUrl(), "confirmed");
  const anchorProvider = new anchor.AnchorProvider(
    connection,
    new anchor.Wallet(getPayer()),
    confirmConfig
  );

  anchor.setProvider(anchorProvider);
  return anchorProvider;
};

export const getLightProvider = async (localTestRelayer?: boolean) => {
  if (!provider) {
    const relayer = await getRelayer(localTestRelayer);

    await setAnchorProvider();

    provider = await Provider.init({
      wallet: readWalletFromFile(),
      relayer,
      url: getRpcUrl(),
      confirmConfig,
      versionedTransactionLookupTable: getLookUpTable(),
    });
    return provider;
  }
  return provider;
};

export const getUser = async ({
  skipFetchBalance,
  localTestRelayer,
}: {
  skipFetchBalance?: boolean;
  localTestRelayer?: boolean;
} = {}): Promise<User> => {
  const provider = await getLightProvider(localTestRelayer);
  const user = await User.init({ provider, skipFetchBalance });
  return user;
};

export const getRelayer = async (localTestRelayer?: boolean) => {
  if (!relayer) {
    if (localTestRelayer) {
      const wallet = readWalletFromFile();
      relayer = new TestRelayer({
        relayerPubkey: wallet.publicKey,
        relayerRecipientSol: wallet.publicKey,
        relayerFee: RELAYER_FEE,
        highRelayerFee: TOKEN_ACCOUNT_FEE,
        payer: wallet,
      });
      return relayer;
    } else {
      relayer = new Relayer(
        getRelayerPublicKey(),
        getRelayerRecipient(),
        RELAYER_FEE,
        TOKEN_ACCOUNT_FEE,
        getRelayerUrl()
      );
    }
  }
  return relayer;
};

type Config = {
  rpcUrl: string;
  relayerUrl: string;
  secretKey: string;
  relayerRecipient: string;
  relayerPublicKey: string;
  payer: string;
  lookUpTable: string;
};

export const getRpcUrl = (): string => {
  const config = getConfig();
  return config.rpcUrl;
};

export const setrpcUrl = (url: string): void => {
  setConfig({ rpcUrl: url });
};

export const getRelayerUrl = (): string => {
  const config = getConfig();
  return config.relayerUrl;
};

export const setRelayerUrl = (url: string): void => {
  setConfig({ relayerUrl: url });
};

export const getSecretKey = () => {
  const config = getConfig();
  return config.secretKey;
};

export const setSecretKey = (key: string) => {
  setConfig({ secretKey: key });
};

export const getRelayerRecipient = () => {
  const config = getConfig();
  return new solana.PublicKey(config.relayerRecipient);
};

export const setRelayerRecipient = (address: string) => {
  setConfig({ relayerRecipient: address });
};

export const getRelayerPublicKey = () => {
  const config = getConfig();
  return new solana.PublicKey(config.relayerPublicKey);
};

export const setRelayerPublicKey = (address: string): void => {
  setConfig({ relayerPublicKey: address });
};

export const getLookUpTable = () => {
  const config = getConfig();

  if (config.rpcUrl.includes(":8899")) {
    console.log("CLI on localhost: creating new LookUpTable");
    return undefined;
  }
  return new solana.PublicKey(config.lookUpTable);
};

export const setLookUpTable = (address: string): void => {
  setConfig({ lookUpTable: address });
};

export const getPayer = () => {
  const secretKey = bs58.decode(getSecretKey());

  let asUint8Array: Uint8Array = new Uint8Array(secretKey);
  let keypair: solana.Keypair = solana.Keypair.fromSecretKey(asUint8Array);

  return keypair;
};

export const setPayer = (key: string) => {
  setConfig({ payer: key });
};

export const getConfig = (filePath?: string): Config => {
  if (!filePath) filePath = process.env.HOME + CONFIG_PATH + CONFIG_FILE_NAME;

  try {
    const data = fs.readFileSync(filePath, "utf-8");
    return JSON.parse(data);
  } catch (error) {
    throw new Error("Failed to read configuration file");
  }

  for (const configPath of pathsToCheck) {
    if (configPath && fs.existsSync(configPath)) {
      try {
        const data = fs.readFileSync(configPath, "utf-8");
        return JSON.parse(data);
      } catch (error) {
        throw new Error(`Failed to read configuration file at ${configPath}`);
      }
    }
  }

  throw new Error("Configuration file not found in the specified paths");
};

export function ensureDirectoryExists(dirPath: string): void {
  if (!fs.existsSync(dirPath)) {
    fs.mkdirSync(dirPath, { recursive: true });
  }
}

export const setConfig = (config: Partial<Config>, filePath?: string): void => {
  if (!filePath) filePath = process.env.HOME + CONFIG_PATH + CONFIG_FILE_NAME;

  // Ensure the directory structure exists
  const dir = path.dirname(filePath);
  if (!fs.existsSync(dir)) {
    fs.mkdirSync(dir, { recursive: true });
  }
  ensureDirectoryExists(process.env.HOME + CONFIG_PATH);
  if (!fs.existsSync(filePath)) {
    let data = {
      ...DEFAULT_CONFIG,
      // TODO: remove this default secret key which we need for tests right now
      secretKey:
        "LsYPAULcTDhjnECes7qhwAdeEUVYgbpX5ri5zijUceTQXCwkxP94zKdG4pmDQmicF7Zbj1AqB44t8qfGE8RuUk8", // bs58.encode(solana.Keypair.generate().secretKey),
    };
    console.log("created file ", filePath);

    fs.writeFileSync(filePath, JSON.stringify(data, null, 2));
    console.log("created file ", filePath);
  }
  try {
    const existingConfig = getConfig();
    const updatedConfig = { ...existingConfig, ...config };
    fs.writeFileSync(filePath, JSON.stringify(updatedConfig, null, 2));
  } catch (error) {
    throw new Error("Failed to update configuration file");
  }
};

export function generateSolanaTransactionURL(
  transactionType: "tx" | "address",
  transactionHash: string,
  cluster: string
): string {
  const url = `https://explorer.solana.com/${transactionType}/${transactionHash}?cluster=${cluster}`;
  return url;
}

export class CustomLoader {
  message: string;
  logInterval: any;
  logTimer: number | null;
  startTime: number;

  constructor(message: string, logInterval = 1000) {
    this.message = message;
    this.logInterval = logInterval;
    this.logTimer = null;
    this.startTime = Date.now();
  }

  start() {
    this.startTime = Date.now();
    const elapsedTime = ((Date.now() - this.startTime) / 1000).toFixed(2);
    process.stdout.write(
      `\n${spinner.dots.frames[Math.floor(Math.random() * 10)]} ${
        this.message
      }\n`
    );
    this.logInterval = setInterval(() => {}, this.logInterval);
  }

  stop(terminateCurve = true) {
    clearInterval(this.logInterval);
    if (terminateCurve) (globalThis as any).curve_bn128.terminate();
    this.logElapsedTime();
  }

  logElapsedTime() {
    const elapsedTime = ((Date.now() - this.startTime) / 1000).toFixed(2);
    process.stdout.write(`\nElapsed time: ${elapsedTime}s\n`);
  }
}

export function isValidURL(url: string): boolean {
  try {
    new URL(url);
    return true;
  } catch (error) {
    return false;
  }
}

export function isValidBase58SecretKey(secretKey: string): boolean {
  const base58Regex =
    /^[123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz]+$/;
  return base58Regex.test(secretKey);
}

export const getConfirmOptions = (flags: any) => {
  if (flags["finalized"]) {
    return ConfirmOptions.finalized;
  } else if (flags["spendable"]) {
    return ConfirmOptions.spendable;
  } else {
    return undefined;
  }
};
