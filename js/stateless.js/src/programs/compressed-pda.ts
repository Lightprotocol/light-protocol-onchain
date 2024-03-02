import { Program, AnchorProvider, setProvider } from "@coral-xyz/anchor";
import {
  PublicKey,
  TransactionInstruction,
  Keypair,
  Connection,
} from "@solana/web3.js";
import { IDL, PspCompressedPda } from "../idls/psp_compressed_pda";
import { confirmConfig, defaultStaticAccounts } from "../constants";
import { useWallet } from "../wallet";
import {
  UtxoWithMerkleContext,
  UtxoWithMerkleProof,
  createUtxo,
} from "../state";
import { toArray } from "../utils/conversion";
import { packInstruction } from "../instruction/pack-instruction";
import { pipe } from "../utils/pipe";
import { placeholderValidityProof } from "../instruction/validity-proof";

export type CompressedTransferParams = {
  /** Utxos with lamports to spend as transaction inputs */
  fromBalance: // TODO: selection upfront
  | UtxoWithMerkleContext
    | UtxoWithMerkleProof
    | (UtxoWithMerkleContext | UtxoWithMerkleProof)[];
  /** Solana Account that will receive transferred compressed lamports as utxo  */
  toPubkey: PublicKey;
  /** Amount of compressed lamports to transfer */
  lamports: number | bigint;
  // TODO: add
  // /** Optional: if different feepayer than owner of utxos */
  // payer?: PublicKey;
};

/**
 * Create compressed account system transaction params
 */
export type CreateCompressedAccountParams = {
  /*
   * Optional utxos with lamports to spend as transaction inputs.
   * Not required unless 'lamports' are specified, as Light doesn't
   * enforce rent on the protocol level.
   */
  fromBalance: UtxoWithMerkleContext[] | UtxoWithMerkleContext;
  /** Public key of the created account */
  newAccountPubkey: PublicKey;
  /** Amount of lamports to transfer to the created compressed account */
  lamports: number | bigint;
  /** Public key of the program or user to assign as the owner of the created compressed account */
  newAccountOwner: PublicKey;
};

export class LightSystemProgram {
  /**
   * @internal
   */
  constructor() {}

  /**
   * Public key that identifies the CompressedPda program
   */
  static programId: PublicKey = new PublicKey(
    // TODO: replace with actual program id
    // can add check to ensure its consistent with the idl
    "42111111111111111111111111111111"
  );

  private static _program: Program<PspCompressedPda> | null = null;

  static get program(): Program<PspCompressedPda> {
    if (!this._program) {
      this.initializeProgram();
    }
    return this._program!;
  }

  /**
   * Initializes the program statically if not already initialized.
   */
  private static initializeProgram() {
    if (!this._program) {
      const mockKeypair = Keypair.generate();
      const mockConnection = new Connection(
        "http://localhost:8899",
        "confirmed"
      );
      const mockProvider = new AnchorProvider(
        mockConnection,
        useWallet(mockKeypair),
        confirmConfig
      );
      setProvider(mockProvider);
      this._program = new Program(IDL, this.programId, mockProvider);
    }
  }

  /**
   * Generate a transaction instruction that transfers compressed
   * lamports from one compressed balance to another solana address
   */
  /// TODO: should just define the createoutput utxo selection + packing
  static async transfer(
    params: CompressedTransferParams
  ): Promise<TransactionInstruction> {
    const recipientUtxo = createUtxo(params.toPubkey, params.lamports);

    // unnecessary if after
    const fromUtxos = pipe(
      toArray<UtxoWithMerkleContext | UtxoWithMerkleProof>,
      coerceIntoUtxoWithMerkleContext
    )(params.fromBalance);

    // TODO: move outside of transfer, selection and (getting merkleproofs and zkp) should happen BEFORE call
    /// find sort utxos by size, then add utxos up until the amount is at least reached, return the selected utxos
    if (new Set(fromUtxos.map((utxo) => utxo.owner.toString())).size > 1) {
      throw new Error("All input utxos must have the same owner");
    }
    const selectedInputUtxos = fromUtxos
      .sort((a, b) => Number(BigInt(a.lamports) - BigInt(b.lamports)))
      .reduce<{
        utxos: (UtxoWithMerkleContext | UtxoWithMerkleProof)[];
        total: bigint;
      }>(
        (acc, utxo) => {
          if (acc.total < params.lamports) {
            acc.utxos.push(utxo);
            acc.total = BigInt(acc.total) + BigInt(utxo.lamports);
          }
          return acc;
        },
        { utxos: [], total: BigInt(0) }
      );

    /// transfer logic
    let changeUtxo;
    const changeAmount = selectedInputUtxos.total - BigInt(params.lamports);
    if (changeAmount > 0) {
      changeUtxo = createUtxo(selectedInputUtxos.utxos[0].owner, changeAmount);
    }

    const outputUtxos = changeUtxo
      ? [recipientUtxo, changeUtxo]
      : [recipientUtxo];

    // TODO: move zkp, merkleproof generation, and rootindices outside of transfer
    const recentValidityProof = placeholderValidityProof();
    const recentInputStateRootIndices = selectedInputUtxos.utxos.map((_) => 0);
    const staticAccounts = defaultStaticAccounts();

    const ix = await packInstruction({
      inputState: coerceIntoUtxoWithMerkleContext(selectedInputUtxos.utxos),
      outputState: outputUtxos,
      recentValidityProof,
      recentInputStateRootIndices,
      payer: selectedInputUtxos.utxos[0].owner, // TODO: dynamic payer,
      staticAccounts,
    });
    return ix;
  }
}

function coerceIntoUtxoWithMerkleContext(
  utxos: (UtxoWithMerkleContext | UtxoWithMerkleProof)[]
): UtxoWithMerkleContext[] {
  return utxos.map((utxo): UtxoWithMerkleContext => {
    if ("merkleProof" in utxo && "rootIndex" in utxo) {
      const { merkleProof, rootIndex, ...rest } = utxo;
      return rest;
    }
    return utxo;
  });
}
