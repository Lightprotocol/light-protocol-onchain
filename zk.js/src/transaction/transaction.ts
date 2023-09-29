import { PublicKey, TransactionInstruction } from "@solana/web3.js";
import { BN, BorshAccountsCoder, Idl, Program, utils } from "@coral-xyz/anchor";
import { Utxo } from "../utxo";
import {
  merkleTreeProgramId,
  TransactionErrorCode,
  TransactionError,
  ProviderErrorCode,
  Provider,
  TransactionParameters,
  firstLetterToUpper,
  createAccountObject,
  firstLetterToLower,
  hashAndTruncateToCircuit,
  MINT,
  AUTHORITY,
  BN_0,
  UTXO_PREFIX_LENGTH,
  N_ASSET_PUBKEYS,
  Account,
} from "../index";
import { IDL_MERKLE_TREE_PROGRAM } from "../idls/index";
import { remainingAccount } from "../types";
import { createAssociatedTokenAccountInstruction } from "@solana/spl-token";
import { getIndices3Dim } from "@lightprotocol/circuit-lib.js";

const path = require("path");

// TODO: make dev provide the classification and check here -> it is easier to check whether transaction parameters are plausible

// add verifier class which is passed in with the constructor
// this class replaces the send transaction, also configures path the proving key and witness, the inputs for the integrity hash
// input custom verifier with three functions by default prepare, proof, send
// include functions from sdk in shieldedTransaction

// TODO: add log option that enables logs

export enum Action {
  SHIELD = "SHIELD",
  TRANSFER = "TRANSFER",
  UNSHIELD = "UNSHIELD",
}

type PublicInputs = {
  root: Array<number>;
  publicAmountSpl: Array<number>;
  txIntegrityHash: Array<number>;
  publicAmountSol: Array<number>;
  publicMintPubkey: Array<number>;
  inputNullifier: Array<Array<number>>;
  outputCommitment: Array<Array<number>>;
  // only for app verifiers
  transactionHash?: Array<number>;
  checkedParams?: Array<Array<number>>;
  publicAppVerifier?: Array<number>;
};

export class Transaction {
  merkleTreeProgram?: Program<typeof IDL_MERKLE_TREE_PROGRAM>;
  shuffleEnabled: Boolean;
  params: TransactionParameters; // contains accounts
  appParams?: any;
  // TODO: relayer shd pls should be part of the provider by default + optional override on Transaction level
  provider: Provider;

  transactionInputs: {
    publicInputs?: PublicInputs;
    rootIndex?: BN;
    proofBytes?: any;
    proofBytesApp?: any;
    publicInputsApp?: any;
    encryptedUtxos?: Uint8Array;
  };

  remainingAccounts?: {
    nullifierPdaPubkeys?: remainingAccount[];
    leavesPdaPubkeys?: remainingAccount[];
    nextTransactionMerkleTree?: remainingAccount;
    nextEventMerkleTree?: remainingAccount;
  };

  proofInput: any;
  firstPath!: string;

  /**
   * Initialize transaction
   *
   * @param relayer recipient of the unshielding
   * @param shuffleEnabled
   */
  constructor({
    provider,
    shuffleEnabled = false,
    params,
    appParams,
  }: {
    provider: Provider;
    shuffleEnabled?: boolean;
    params: TransactionParameters;
    appParams?: any;
  }) {
    if (!provider)
      throw new TransactionError(
        TransactionErrorCode.PROVIDER_UNDEFINED,
        "constructor",
      );
    if (!provider.poseidon)
      throw new TransactionError(
        TransactionErrorCode.POSEIDON_HASHER_UNDEFINED,
        "constructor",
        "Poseidon hasher in provider undefined.",
      );
    if (!provider.solMerkleTree)
      throw new TransactionError(
        ProviderErrorCode.SOL_MERKLE_TREE_UNDEFINED,
        "constructor",
        "Merkle tree not set in provider",
      );
    if (!provider.wallet)
      throw new TransactionError(
        TransactionErrorCode.WALLET_UNDEFINED,
        "constructor",
        "Wallet not set in provider.",
      );
    if (!params) {
      throw new TransactionError(
        TransactionErrorCode.TX_PARAMETERS_UNDEFINED,
        "constructor",
      );
    }
    if (!params.verifierIdl)
      throw new TransactionError(
        TransactionErrorCode.VERIFIER_IDL_UNDEFINED,
        "constructor",
        "",
      );

    if (params.verifierConfig.in.toString() === "4" && !appParams)
      throw new TransactionError(
        TransactionErrorCode.APP_PARAMETERS_UNDEFINED,
        "constructor",
        "For application transactions application parameters need to be specified.",
      );

    if (appParams && params.verifierConfig.in.toString() !== "4")
      throw new TransactionError(
        TransactionErrorCode.INVALID_VERIFIER_SELECTED,
        "constructor",
        "For application transactions, an application-enabled verifier (like verifier two) is required.",
      );
    this.provider = provider;

    this.shuffleEnabled = shuffleEnabled;
    // TODO: create and check for existence of merkleTreeAssetPubkey depending on utxo asset
    this.params = params;
    this.appParams = appParams;

    //TODO: change to check whether browser/node wallet are the same as signing address
    if (params.action === Action.SHIELD) {
      let wallet = this.provider.wallet;
      if (
        wallet?.publicKey.toBase58() !==
          params.relayer.accounts.relayerPubkey.toBase58() &&
        wallet?.publicKey.toBase58() !==
          params.accounts.signingAddress?.toBase58()
      ) {
        throw new TransactionError(
          TransactionErrorCode.WALLET_RELAYER_INCONSISTENT,
          "constructor",
          "The wallet used in your Node.js or Browser environment does not match the wallet used for the senderFee when setting up the relayer during the deposit process. They need to be the same.",
        );
      }
    }

    this.transactionInputs = {};
    this.remainingAccounts = {};
  }

  async compileAndProve(account: Account) {
    await this.compile(account);
    if (!this.params)
      throw new TransactionError(
        TransactionErrorCode.TX_PARAMETERS_UNDEFINED,
        "compileAndProve",
      );
    await this.getProof(account);
    if (this.appParams) await this.getAppProof(account);

    const { rootIndex, remainingAccounts } = await this.provider.getRootIndex();
    this.transactionInputs.rootIndex = rootIndex;
    this.remainingAccounts = {
      ...this.remainingAccounts,
      ...remainingAccounts,
    };
    this.getPdaAddresses();
    return this.getInstructions(this.appParams ? this.appParams : this.params);
  }

  /**
   * @description Prepares proof inputs.
   */
  async compile(account: Account) {
    this.firstPath = path.resolve(__dirname, "../../build-circuits/");

    this.shuffleUtxos(this.params.inputUtxos);
    this.shuffleUtxos(this.params.outputUtxos);

    if (!this.provider.solMerkleTree)
      throw new TransactionError(
        ProviderErrorCode.SOL_MERKLE_TREE_UNDEFINED,
        "getProofInput",
      );
    await this.params.getTxIntegrityHash(this.provider.poseidon);
    if (!this.params.txIntegrityHash)
      throw new TransactionError(
        TransactionErrorCode.TX_INTEGRITY_HASH_UNDEFINED,
        "compile",
      );

    const { inputMerklePathIndices, inputMerklePathElements } =
      this.provider.solMerkleTree.getMerkleProofs(
        this.provider,
        this.params.inputUtxos,
      );

    this.proofInput = {
      root: this.provider.solMerkleTree.merkleTree.root(),
      inputNullifier: this.params.inputUtxos.map((x) =>
        x.getNullifier({ poseidon: this.provider.poseidon, account }),
      ),
      publicAmountSpl: this.params.publicAmountSpl.toString(),
      publicAmountSol: this.params.publicAmountSol.toString(),
      publicMintPubkey: this.getMint(),
      inPathIndices: inputMerklePathIndices,
      inPathElements: inputMerklePathElements,
      internalTxIntegrityHash: this.params.txIntegrityHash.toString(),
      transactionVersion: "0",
      txIntegrityHash: this.params.txIntegrityHash.toString(),
      outputCommitment: this.params.outputUtxos.map((x) =>
        x.getCommitment(this.provider.poseidon),
      ),
      inAmount: this.params.inputUtxos?.map((x) => x.amounts),
      inBlinding: this.params.inputUtxos?.map((x) => x.blinding),
      assetPubkeys: this.params.assetPubkeysCircuit,
      outAmount: this.params.outputUtxos?.map((x) => x.amounts),
      outBlinding: this.params.outputUtxos?.map((x) => x.blinding),
      outPubkey: this.params.outputUtxos?.map((x) => x.publicKey),
      inIndices: getIndices3Dim(
        this.params.inputUtxos[0].assets.length,
        N_ASSET_PUBKEYS,
        this.params.inputUtxos.map((utxo) => utxo.assetsCircuit),
        this.params.assetPubkeysCircuit,
      ),
      outIndices: getIndices3Dim(
        this.params.inputUtxos[0].assets.length,
        N_ASSET_PUBKEYS,
        this.params.outputUtxos.map((utxo) => utxo.assetsCircuit),
        this.params.assetPubkeysCircuit,
      ),
      inAppDataHash: this.params.inputUtxos?.map((x) => x.appDataHash),
      outAppDataHash: this.params.outputUtxos?.map((x) => x.appDataHash),
      inPoolType: this.params.inputUtxos?.map((x) => x.poolType),
      outPoolType: this.params.outputUtxos?.map((x) => x.poolType),
      inVerifierPubkey: this.params.inputUtxos?.map(
        (x) => x.verifierAddressCircuit,
      ),
      outVerifierPubkey: this.params.outputUtxos?.map(
        (x) => x.verifierAddressCircuit,
      ),
    };

    if (this.appParams) {
      this.proofInput.transactionHash = this.params.getTransactionHash(
        this.provider.poseidon,
      );

      this.proofInput.publicAppVerifier = hashAndTruncateToCircuit(
        TransactionParameters.getVerifierProgramId(
          this.appParams.verifierIdl,
        ).toBuffer(),
      );

      this.proofInput = {
        ...this.appParams.inputs,
        ...this.proofInput,
        inPublicKey: this.params?.inputUtxos?.map((utxo) => utxo.publicKey),
      };
    }
  }

  getMint() {
    if (this.params.publicAmountSpl.eq(BN_0)) {
      return BN_0;
    } else if (this.params.assetPubkeysCircuit) {
      return this.params.assetPubkeysCircuit[1];
    } else {
      throw new TransactionError(
        TransactionErrorCode.GET_MINT_FAILED,
        "getMint",
        "Failed to retrieve mint. The transaction parameters should contain 'assetPubkeysCircuit' after initialization, but it's missing.",
      );
    }
  }

  async getProof(account: Account) {
    const { parsedProof, parsedPublicInputsObject } =
      await account.getProofInternal(
        this.firstPath,
        this.params,
        this.proofInput,
        true,
      );
    this.transactionInputs.proofBytes = parsedProof;
    this.transactionInputs.publicInputs = parsedPublicInputsObject as any;
  }

  async getAppProof(account: Account) {
    if (!this.appParams)
      throw new TransactionError(
        TransactionErrorCode.APP_PARAMETERS_UNDEFINED,
        "getAppProof",
      );
    if (!this.appParams.path)
      throw new TransactionError(
        TransactionErrorCode.FIRST_PATH_APP_UNDEFINED,
        "getAppProof",
        "The app path is not defined. Please ensure it is specified in 'appParams'.",
      );

    const res = await account.getProofInternal(
      this.appParams.path,
      this.appParams,
      this.proofInput,
      true,
    );
    this.transactionInputs.proofBytesApp = {
      proofAApp: res.parsedProof.proofA,
      proofBApp: res.parsedProof.proofB,
      proofCApp: res.parsedProof.proofC,
    };
    this.transactionInputs.publicInputsApp = res.parsedPublicInputsObject;
  }

  static getSignerAuthorityPda(
    merkleTreeProgramId: PublicKey,
    verifierProgramId: PublicKey,
  ) {
    return PublicKey.findProgramAddressSync(
      [merkleTreeProgramId.toBytes()],
      verifierProgramId,
    )[0];
  }

  static getRegisteredVerifierPda(
    merkleTreeProgramId: PublicKey,
    verifierProgramId: PublicKey,
  ): PublicKey {
    return PublicKey.findProgramAddressSync(
      [verifierProgramId.toBytes()],
      merkleTreeProgramId,
    )[0];
  }

  /**
   * Asynchronously generates an array of transaction instructions based on the provided transaction parameters.
   *
   * 1. Validates that the required properties of transactionInputs and verifier are defined.
   * 2. Retrieves ordered instruction names from the verifier program by:
   *    a. Filtering instructions based on a suffix pattern (e.g., "First", "Second", "Third", etc.).
   *    b. Sorting instructions according to the order of suffixes.
   * 3. Constructs an input object containing the necessary data for encoding.
   * 4. Iterates through the instruction names, encoding the inputs and generating transaction instructions.
   * 5. Returns an array of generated transaction instructions.
   *
   * @param {TransactionParameters} params - Object containing the required transaction parameters.
   * @returns {Promise<TransactionInstruction[]>} - Promise resolving to an array of generated transaction instructions.
   */
  async getInstructions(
    params: TransactionParameters,
  ): Promise<TransactionInstruction[]> {
    const verifierProgram = TransactionParameters.getVerifierProgram(
      params.verifierIdl,
      this.provider.provider,
    );

    if (!this.transactionInputs.publicInputs)
      throw new TransactionError(
        TransactionErrorCode.PUBLIC_INPUTS_UNDEFINED,
        "getInstructions",
      );
    if (!verifierProgram)
      throw new TransactionError(
        TransactionErrorCode.VERIFIER_PROGRAM_UNDEFINED,
        "getInstructions",
      );

    const getOrderedInstructionNames = (verifierIdl: Idl) => {
      const orderedInstructionNames = verifierIdl.instructions
        .filter((instruction) =>
          /First|Second|Third|Fourth|Fifth|Sixth|Seventh|Eighth|Ninth/.test(
            instruction.name,
          ),
        )
        .sort((a, b) => {
          const suffixes = [
            "First",
            "Second",
            "Third",
            "Fourth",
            "Fifth",
            "Sixth",
            "Seventh",
            "Eighth",
            "Ninth",
          ];
          const aIndex = suffixes.findIndex((suffix) =>
            a.name.endsWith(suffix),
          );
          const bIndex = suffixes.findIndex((suffix) =>
            b.name.endsWith(suffix),
          );

          if (aIndex === 7 || bIndex === 7) {
            throw new Error("Found an instruction with the 'Eighth' suffix.");
          }

          return aIndex - bIndex;
        })
        .map((instruction) => instruction.name);

      return orderedInstructionNames;
    };

    if (
      this.params.verifierConfig.out == 2 &&
      this.params.encryptedUtxos &&
      this.params.encryptedUtxos
        .slice(240 + UTXO_PREFIX_LENGTH * 2)
        .some((el) => el !== 0)
    ) {
      this.params.encryptedUtxos = this.params.encryptedUtxos.slice(
        0,
        240 + UTXO_PREFIX_LENGTH * 2,
      );
    }
    let inputObject = {
      message: this.params.message,
      ...this.transactionInputs.proofBytes,
      ...this.transactionInputs.publicInputs,
      rootIndex: this.transactionInputs.rootIndex,
      relayerFee: this.params.relayer.getRelayerFee(this.params.ataCreationFee),
      encryptedUtxos: Buffer.from(this.params.encryptedUtxos!),
    };
    if (this.appParams) {
      inputObject = {
        ...inputObject,
        ...this.appParams.inputs,
        ...this.transactionInputs.proofBytesApp,
        ...this.transactionInputs.publicInputsApp,
      };
    }

    let instructions = [];
    // TODO: make mint dynamic
    /**
     * Problem:
     * - for spl withdrawals we need an initialized associated token we can withdraw to
     * - this transaction needs to be signed by the owner of the associated token account? has it?
     */
    if (this.params.ataCreationFee) {
      if (!this.params.accounts.recipientSpl)
        throw new TransactionError(
          TransactionErrorCode.SPL_RECIPIENT_UNDEFINED,
          "getInstructions",
          "Probably sth in the associated token address generation went wrong",
        );
      if (!this.params.accounts.recipientSol)
        throw new TransactionError(
          TransactionErrorCode.SPL_RECIPIENT_UNDEFINED,
          "getInstructions",
          "Probably sth in the associated token address generation went wrong",
        );
      let ix = createAssociatedTokenAccountInstruction(
        this.params.relayer.accounts.relayerPubkey,
        this.params.accounts.recipientSpl,
        this.params.accounts.recipientSol,
        MINT,
      );
      instructions.push(ix);
    }

    const instructionNames = getOrderedInstructionNames(params.verifierIdl);
    for (let i = 0; i < instructionNames.length; i++) {
      const instruction = instructionNames[i];
      const coder = new BorshAccountsCoder(params.verifierIdl);

      const accountName = "instructionData" + firstLetterToUpper(instruction);
      let inputs = createAccountObject(
        inputObject,
        params.verifierIdl.accounts!,
        accountName,
      );

      let inputsVec = (await coder.encode(accountName, inputs)).subarray(8);
      // TODO: check whether app account names overlap with system account names and throw an error if so
      let appAccounts = {};
      if (this.appParams?.accounts) {
        appAccounts = this.appParams.accounts;
      }
      const methodName = firstLetterToLower(instruction);
      const method = verifierProgram.methods[
        methodName as keyof typeof verifierProgram.methods
      ](inputsVec).accounts({
        ...this.params.accounts,
        ...this.params.relayer.accounts,
        ...appAccounts,
        relayerRecipientSol:
          this.params.action === Action.SHIELD
            ? AUTHORITY
            : this.params.relayer.accounts.relayerRecipientSol,
      });

      // Check if it's the last iteration
      if (i === instructionNames.length - 1) {
        let remainingAccounts = [
          ...this.remainingAccounts!.nullifierPdaPubkeys!,
          ...this.remainingAccounts!.leavesPdaPubkeys!,
        ];
        if (this.remainingAccounts!.nextTransactionMerkleTree !== undefined) {
          remainingAccounts.push(
            this.remainingAccounts!.nextTransactionMerkleTree,
          );
        }
        if (this.remainingAccounts!.nextEventMerkleTree !== undefined) {
          remainingAccounts.push(this.remainingAccounts!.nextEventMerkleTree);
        }
        method.remainingAccounts(remainingAccounts);
      }

      const ix = await method.instruction();

      instructions?.push(ix);
    }
    return instructions;
  }

  getPdaAddresses() {
    if (!this.transactionInputs.publicInputs)
      throw new TransactionError(
        TransactionErrorCode.PUBLIC_INPUTS_UNDEFINED,
        "getPdaAddresses",
        "",
      );
    if (!this.params.verifierIdl)
      throw new TransactionError(
        TransactionErrorCode.VERIFIER_IDL_UNDEFINED,
        "getPdaAddresses",
        "",
      );
    if (!this.params.relayer)
      throw new TransactionError(
        TransactionErrorCode.RELAYER_UNDEFINED,
        "getPdaAddresses",
        "",
      );
    if (!this.remainingAccounts)
      throw new TransactionError(
        TransactionErrorCode.REMAINING_ACCOUNTS_NOT_CREATED,
        "getPdaAddresses",
        "Remaining accounts undefined",
      );

    let nullifiers = this.transactionInputs.publicInputs.inputNullifier;
    let signer = this.params.relayer.accounts.relayerPubkey;

    this.remainingAccounts.nullifierPdaPubkeys = [];
    for (const i in nullifiers) {
      this.remainingAccounts.nullifierPdaPubkeys.push({
        isSigner: false,
        isWritable: true,
        pubkey: Transaction.getNullifierPdaPublicKey(
          nullifiers[i],
          merkleTreeProgramId,
        ),
      });
    }

    this.remainingAccounts.leavesPdaPubkeys = [];

    for (
      let j = 0;
      j < this.transactionInputs.publicInputs.outputCommitment.length;
      j += 2
    ) {
      this.remainingAccounts.leavesPdaPubkeys.push({
        isSigner: false,
        isWritable: true,
        pubkey: PublicKey.findProgramAddressSync(
          [
            Buffer.from(
              Array.from(
                this.transactionInputs.publicInputs.outputCommitment[j],
              ).reverse(),
            ),
            utils.bytes.utf8.encode("leaves"),
          ],
          merkleTreeProgramId,
        )[0],
      });
    }

    if (this.appParams) {
      this.params.accounts.verifierState = PublicKey.findProgramAddressSync(
        [signer.toBytes(), utils.bytes.utf8.encode("VERIFIER_STATE")],
        TransactionParameters.getVerifierProgramId(this.appParams.verifierIdl),
      )[0];
    } else {
      this.params.accounts.verifierState = PublicKey.findProgramAddressSync(
        [signer.toBytes(), utils.bytes.utf8.encode("VERIFIER_STATE")],
        this.params.verifierProgramId,
      )[0];
    }
  }

  static getNullifierPdaPublicKey(
    nullifier: number[],
    merkleTreeProgramId: PublicKey,
  ) {
    return PublicKey.findProgramAddressSync(
      [Uint8Array.from([...nullifier]), utils.bytes.utf8.encode("nf")],
      merkleTreeProgramId,
    )[0];
  }

  // TODO: use higher entropy rnds
  shuffleUtxos(utxos: Utxo[]) {
    if (!this.shuffleEnabled) {
      return;
    }
    let currentIndex: number = utxos.length;
    let randomIndex: number;

    // While there remain elements to shuffle...
    while (0 !== currentIndex) {
      // Pick a remaining element...
      randomIndex = Math.floor(Math.random() * currentIndex);
      currentIndex--;

      // And swap it with the current element.
      [utxos[currentIndex], utxos[randomIndex]] = [
        utxos[randomIndex],
        utxos[currentIndex],
      ];
    }

    return utxos;
  }

  static getTokenAuthority(): PublicKey {
    return PublicKey.findProgramAddressSync(
      [utils.bytes.utf8.encode("spl")],
      merkleTreeProgramId,
    )[0];
  }
}
