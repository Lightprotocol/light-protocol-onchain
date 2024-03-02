import { PublicKey } from "@solana/web3.js";
import { LightWasm } from "@lightprotocol/account.rs";
import {
  Utxo,
  UtxoWithMerkleContext,
  addMerkleContextToUtxo,
  createUtxo,
} from "./utxo";
import { TlvSerial, deserializeTlv, serializeTlv } from "./utxo-data";
import { bufToDecStr, hashToBn254FieldSizeLe } from "../utils/conversion";
import { bigint254 } from "./bigint254";

export type InputUtxoSerial = {
  owner: number;
  leafIndex: number;
  lamports: number;
  data: TlvSerial | null;
};

export type OutputUtxoSerial = {
  owner: number;
  lamports: number;
  data: TlvSerial | null;
};

export class UtxoSerde {
  pubkeyArray: PublicKey[];
  u64Array: bigint[];
  inputUtxos: [InputUtxoSerial, number, number][];
  outputUtxos: [OutputUtxoSerial, number][];

  constructor() {
    this.pubkeyArray = [];
    this.u64Array = [];
    this.inputUtxos = [];
    this.outputUtxos = [];
  }
  addinputUtxos(
    utxosToAdd: Utxo[],
    accounts: PublicKey[],
    leafIndices: number[],
    inputUtxoMerkleTreePubkeys: PublicKey[],
    nullifierArrayPubkeys: PublicKey[]
  ): UtxoSerde {
    if (this.inputUtxos.length > 0) {
      throw new Error("inputUtxosAlreadyAdded");
    }
    if (
      utxosToAdd.length !== leafIndices.length ||
      utxosToAdd.length !== inputUtxoMerkleTreePubkeys.length ||
      utxosToAdd.length !== nullifierArrayPubkeys.length
    ) {
      throw new Error("ArrayLengthMismatch");
    }

    const utxos: [InputUtxoSerial, number, number][] = [];
    const merkleTreeIndices = new Map<string, number>();
    const nullifierIndices = new Map<string, number>();

    utxosToAdd.forEach((utxo, i) => {
      const ownerIndex = accounts.findIndex((acc) => acc.equals(utxo.owner));
      const owner =
        ownerIndex >= 0
          ? ownerIndex
          : this.pubkeyArray.push(utxo.owner) - 1 + accounts.length;
      const lamportsIndex = this.u64Array.findIndex((l) => l === utxo.lamports);
      const lamports =
        lamportsIndex >= 0
          ? lamportsIndex
          : this.u64Array.push(BigInt(utxo.lamports)) - 1;

      const data = utxo.data
        ? serializeTlv(utxo.data, this.pubkeyArray, accounts)
        : null;

      const inputUtxoSerializable: InputUtxoSerial = {
        owner,
        leafIndex: leafIndices[i],
        lamports,
        data,
      };

      // Calculate indices for merkle tree and nullifier array pubkeys
      let merkleTreeIndex = merkleTreeIndices.get(
        inputUtxoMerkleTreePubkeys[i].toString()
      );
      if (merkleTreeIndex === undefined) {
        merkleTreeIndex = merkleTreeIndices.size;
        merkleTreeIndices.set(
          inputUtxoMerkleTreePubkeys[i].toString(),
          merkleTreeIndex
        );
      }

      let nullifierIndex = nullifierIndices.get(
        nullifierArrayPubkeys[i].toString()
      );
      if (nullifierIndex === undefined) {
        nullifierIndex = nullifierIndices.size;
        nullifierIndices.set(
          nullifierArrayPubkeys[i].toString(),
          nullifierIndex
        );
      }

      utxos.push([inputUtxoSerializable, merkleTreeIndex, nullifierIndex]);
    });

    // Extend SerializedUtxos
    this.inputUtxos.push(...utxos);
    return this;
  }

  addoutputUtxos(
    utxosToAdd: Utxo[],
    accounts: PublicKey[],
    remainingAccountsPubkeys: PublicKey[],
    outputUtxoMerkleTreePubkeys: PublicKey[]
  ): UtxoSerde {
    if (utxosToAdd.length === 0) return this;

    const utxos: [OutputUtxoSerial, number][] = [];
    const merkleTreeIndices = new Map<string, number>();
    const remainingAccountsIndices = new Map<string, number>();

    // Initialize indices for remaining accounts pubkeys
    remainingAccountsPubkeys.forEach((pubkey, index) => {
      remainingAccountsIndices.set(pubkey.toString(), index);
    });

    utxosToAdd.forEach((utxo, i) => {
      const ownerIndex = accounts.findIndex((acc) => acc.equals(utxo.owner));
      const owner =
        ownerIndex >= 0
          ? ownerIndex
          : this.pubkeyArray.findIndex((pubkey) => pubkey.equals(utxo.owner)) >=
            0
          ? this.pubkeyArray.findIndex((pubkey) => pubkey.equals(utxo.owner)) +
            accounts.length
          : this.pubkeyArray.push(utxo.owner) - 1 + accounts.length;
      const lamportsIndex = this.u64Array.findIndex(
        (l) => l === BigInt(utxo.lamports)
      );
      const lamports =
        lamportsIndex >= 0
          ? lamportsIndex
          : this.u64Array.push(BigInt(utxo.lamports)) - 1;

      const data = utxo.data
        ? serializeTlv(utxo.data, this.pubkeyArray, accounts)
        : null;

      const outputUtxoSerializable: OutputUtxoSerial = {
        owner,
        lamports,
        data,
      };

      // Calc index for merkle tree pubkey
      let merkleTreeIndex = merkleTreeIndices.get(
        outputUtxoMerkleTreePubkeys[i].toString()
      );
      if (merkleTreeIndex === undefined) {
        merkleTreeIndex = remainingAccountsIndices.get(
          outputUtxoMerkleTreePubkeys[i].toString()
        );
        if (merkleTreeIndex === undefined) {
          merkleTreeIndex =
            remainingAccountsIndices.size + merkleTreeIndices.size;
          merkleTreeIndices.set(
            outputUtxoMerkleTreePubkeys[i].toString(),
            merkleTreeIndex
          );
        }
      }

      utxos.push([outputUtxoSerializable, merkleTreeIndex]);
    });

    // Extend SerializedUtxos
    this.outputUtxos.push(...utxos);
    return this;
  }

  async deserializeInputUtxos(
    hasher: LightWasm,
    accounts: PublicKey[],
    merkleTreeAccounts: PublicKey[],
    stateNullifierQueues: PublicKey[]
  ): Promise<UtxoWithMerkleContext[]> {
    const inputUtxos: UtxoWithMerkleContext[] = [];

    this.inputUtxos.forEach(async (inputUtxoSerialized, i) => {
      const [inputUtxo, merkleTreeAccountIndex, stateNullifierQueueIndex] =
        inputUtxoSerialized;

      // resolve owner
      const owner =
        inputUtxo.owner < accounts.length
          ? accounts[inputUtxo.owner]
          : this.pubkeyArray[inputUtxo.owner - accounts.length];

      // resolve lamports
      const lamports = this.u64Array[inputUtxo.lamports];

      // resolve data
      const data = inputUtxo.data
        ? deserializeTlv(inputUtxo.data, [...accounts, ...this.pubkeyArray])
        : undefined;

      // reconstruct inputUtxo
      const utxo = createUtxo(owner, lamports, data);
      const utxoHash = await createUtxoHash(
        hasher,
        utxo,
        merkleTreeAccounts[merkleTreeAccountIndex],
        inputUtxo.leafIndex
      );
      const utxoWithMerkleContext = addMerkleContextToUtxo(
        utxo,
        utxoHash,
        merkleTreeAccounts[merkleTreeAccountIndex],
        inputUtxo.leafIndex,
        stateNullifierQueues[stateNullifierQueueIndex]
      );

      inputUtxos.push(utxoWithMerkleContext);
    });

    return inputUtxos;
  }

  deserializeOutputUtxos(accounts: PublicKey[]): [Utxo, number][] {
    const outputUtxos: [Utxo, number][] = [];

    for (const [outputUtxoSerialized, indexMerkleTreeAccount] of this
      .outputUtxos) {
      // Resolve owner
      const owner =
        outputUtxoSerialized.owner < accounts.length
          ? accounts[outputUtxoSerialized.owner]
          : this.pubkeyArray[outputUtxoSerialized.owner - accounts.length];

      // Resolve lamports
      const lamports = this.u64Array[outputUtxoSerialized.lamports];

      // Resolve data
      const data = outputUtxoSerialized.data
        ? deserializeTlv(outputUtxoSerialized.data, [
            ...accounts,
            ...this.pubkeyArray,
          ])
        : undefined;

      // Reconstruct Utxo
      const utxo = createUtxo(owner, lamports, data);

      outputUtxos.push([utxo, indexMerkleTreeAccount]);
    }

    return outputUtxos;
  }
}

/// TODO: bunch of redundant conversions. optimize.
/** Computes unique utxo value from merkleTree, leafIndex */
const computeBlinding = async (
  hasher: LightWasm,
  merkleTreePublicKey: PublicKey,
  leafIndex: number
): Promise<bigint254> => {
  /// ensure <254-bit
  const mtHash = await hashToBn254FieldSizeLe(merkleTreePublicKey.toBuffer());
  if (!mtHash) throw new Error("Failed to hash merkle tree public key");

  const mtPubkeyDecStr = bufToDecStr(mtHash[0]);
  const leafIndexDecStr = BigInt(leafIndex).toString();

  return hasher.poseidonHashBigint([mtPubkeyDecStr, leafIndexDecStr]);
};

/// TODO: bunch of redundant conversions. optimize.
/**
 * Hashes a UTXO preimage.
 * Hash inputs: owner, blinding(merkleTree,leafIndex), lamports, tlvDataHash
 *
 * async for browser crypto.digest support */
async function createUtxoHash(
  hasher: LightWasm,
  utxo: Utxo,
  merkleTree: PublicKey,
  leafIndex: number
): Promise<bigint254> {
  const { owner, lamports, data } = utxo;

  /// hash all tlv elements into a single hash
  const tlvDataHash = data
    ? hasher.poseidonHashString(data.map((d) => d.dataHash.toString()))
    : BigInt(0).toString();

  /// ensure <254-bit
  const ownerHash = await hashToBn254FieldSizeLe(owner.toBuffer());
  if (!ownerHash) throw new Error("Failed to hash owner public key");
  const ownerDecStr = bufToDecStr(ownerHash[0]);

  const lamportsDecStr = BigInt(lamports).toString();

  const blindingDecStr = (
    await computeBlinding(hasher, merkleTree, leafIndex)
  ).toString();

  return hasher.poseidonHashBigint([
    ownerDecStr,
    blindingDecStr,
    lamportsDecStr,
    tlvDataHash,
  ]);
}
