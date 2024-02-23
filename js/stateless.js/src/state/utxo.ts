import { PublicKey } from "@solana/web3.js";
import { bigint254 } from "./bigint254";
import { TlvDataElement } from "./utxo-data";

/** Describe the generic utxo details applicable to every utxo. */
export type Utxo = {
  /** Public key of program or user that owns the utxo */
  owner: PublicKey;
  /** Optional lamports attached to the utxo */
  lamports: bigint;
  /** Optional data attached to the utxo */
  data: TlvDataElement[];
  /**
   * TODO: Implement address functionality
   * Optional unique account ID that is persistent across transactions.
   */
  address?: PublicKey;
};

/** Context for utxos inserted into a state Merkle tree */
export type MerkleContext = {
  /** Poseidon hash of the utxo preimage  */
  hash: bigint254;
  /** State Merkle tree ID */
  merkletreeId: bigint;
  /** 'hash' position within the Merkle tree */
  leafIndex: bigint;
  /** Recent valid 'hash' proof path, expiring after n slots */
  merkleProof?: string[];
};

/** Utxo with Merkle tree context */
export type UtxoWithMerkleContext = Utxo & MerkleContext;

/** Utxo object factory */
export const createUtxo = (
  owner: PublicKey,
  lamports: bigint,
  data: TlvDataElement[],
  address?: PublicKey,
  merkleContext?: MerkleContext
): Utxo | UtxoWithMerkleContext => ({
  owner,
  lamports,
  data,
  address,
  ...merkleContext,
});

/** Add Merkle tree context to a utxo */
export const addMerkleContextToUtxo = (
  utxo: Utxo,
  hash: bigint254,
  merkletreeId: bigint,
  leafIndex: bigint,
  merkleProof?: string[]
): UtxoWithMerkleContext => ({
  ...utxo,
  leafIndex,
  hash,
  merkletreeId,
  merkleProof,
});

/** Append a merkle proof to a utxo */
export const addMerkleProofToUtxo = (
  utxo: UtxoWithMerkleContext,
  proof: string[]
): UtxoWithMerkleContext => ({
  ...utxo,
  merkleProof: proof,
});

/** Factory for TLV data elements */
export const createTlvDataElement = (
  discriminator: Uint8Array,
  owner: PublicKey,
  data: Uint8Array,
  dataHash: Uint8Array
): TlvDataElement => ({
  discriminator,
  owner,
  data,
  dataHash,
});

/** Filter utxos with compressed lamports. Excludes PDAs and token accounts */
export function getCompressedSolUtxos(utxos: Utxo[]): Utxo[] {
  return utxos.filter(
    (utxo) => utxo.lamports > BigInt(0) && utxo.data.length === 0
  );
}
