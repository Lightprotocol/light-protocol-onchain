import { useState, useEffect } from "react";
import { PublicKey } from "@solana/web3.js";

const isSolanaPublicKey = (string: string): boolean => {
  try {
    if (PublicKey.isOnCurve(string)) {
      new PublicKey(string);
      return true;
    }
    console.log("!isOnCurve");
    return false;
  } catch (err) {
    console.log("!Pubkey");
    return false;
  }
};

export function useSendType(recipient: string) {
  const [isUnshield, setIsUnshield] = useState(false);

  useEffect(() => {
    if (recipient) {
      setIsUnshield(isSolanaPublicKey(recipient));
    }
  }, [recipient]);

  return isUnshield;
}
