import { Keypair, Commitment } from "@solana/web3.js";
import { Wallet } from "./interface";

export const useWallet = (
  keypair: Keypair,
  url: string = "http://127.0.0.1:8899",
  commitment: Commitment = "confirmed"
  //   isNodeWallet: boolean = true,
) => {
  url = url !== "mock" ? url : "http://127.0.0.1:8899";
  const wallet = new Wallet(keypair, url, commitment);
  return {
    publicKey: wallet._publicKey,
    sendAndConfirmTransaction: wallet.sendAndConfirmTransaction,
    signMessage: wallet.signMessage,
    signTransaction: wallet.signTransaction,
    signAllTransactions: wallet.signAllTransactions,
    sendTransaction: wallet.sendTransaction,
    getProof: wallet.getProof,
    // isNodeWallet,
  };
};
