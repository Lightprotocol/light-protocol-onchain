"use strict";
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || function (mod) {
    if (mod && mod.__esModule) return mod;
    var result = {};
    if (mod != null) for (var k in mod) if (k !== "default" && Object.prototype.hasOwnProperty.call(mod, k)) __createBinding(result, mod, k);
    __setModuleDefault(result, mod);
    return result;
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.fetchRecentTransactions = exports.findMatchingInstruction = exports.getUserIndexTransactions = exports.TransactionIndexerEvent = void 0;
const web3_js_1 = require("@solana/web3.js");
const anchor_1 = require("@coral-xyz/anchor");
const borsh = __importStar(require("@coral-xyz/borsh"));
const spl_account_compression_1 = require("@solana/spl-account-compression");
const bytes_1 = require("@coral-xyz/anchor/dist/cjs/utils/bytes");
const constants_1 = require("../constants");
const transaction_1 = require("./transaction");
const utils_1 = require("../utils");
class TransactionIndexerEvent {
    constructor() {
        this.borshSchema = borsh.struct([
            borsh.vec(borsh.array(borsh.u8(), 32), "leaves"),
            borsh.array(borsh.u8(), 32, "publicAmountSpl"),
            borsh.array(borsh.u8(), 32, "publicAmountSol"),
            borsh.u64("relayerFee"),
            borsh.vec(borsh.u8(), "encryptedUtxos"),
            borsh.vec(borsh.array(borsh.u8(), 32), "nullifiers"),
            borsh.u64("firstLeafIndex"),
            borsh.vecU8("message"),
        ]);
    }
    deserialize(buffer) {
        try {
            return this.borshSchema.decode(buffer);
        }
        catch (e) {
            return null;
        }
    }
}
exports.TransactionIndexerEvent = TransactionIndexerEvent;
/**
 *  Call Flow:
 *  fetchRecentTransactions() <-- called in indexer
 *    getTransactionsBatch()
 *      getSigsForAdd()
 *		    getTxForSig()
 *		      make Events:
 *			    parseTransactionEvents()
 *			    enrichParsedTransactionEvents()
 */
/**
 * @async
 * @description This functions takes the IndexedTransaction and spentUtxos of user any return the filtered user indexed transactions
 * @function getUserIndexTransactions
 * @param {IndexedTransaction[]} indexedTransactions - An array to which the processed transaction data will be pushed.
 * @param {Provider} provider - provider class
 * @param {spentUtxos} Utxo[] - The array of user spentUtxos
 * @returns {Promise<void>}
 */
const getUserIndexTransactions = async (indexedTransactions, provider, tokenBalances) => {
    const transactionHistory = [];
    const spentUtxos = (0, utils_1.getUpdatedSpentUtxos)(tokenBalances);
    indexedTransactions.forEach((trx) => {
        const nullifierZero = new anchor_1.BN(trx.nullifiers[0]).toString();
        const nullifierOne = new anchor_1.BN(trx.nullifiers[1]).toString();
        const isFromUser = trx.signer.toBase58() === provider.wallet.publicKey.toBase58();
        const inSpentUtxos = [];
        const outSpentUtxos = [];
        spentUtxos?.forEach((sUtxo) => {
            const matchesNullifier = sUtxo._nullifier === nullifierOne || sUtxo._nullifier === nullifierZero;
            let matchesCommitment = false;
            for (const leaf of trx.leaves) {
                if (!matchesCommitment) {
                    matchesCommitment =
                        sUtxo._commitment === new anchor_1.BN(leaf, "le").toString();
                }
            }
            if (matchesNullifier) {
                inSpentUtxos.push(sUtxo);
            }
            if (matchesCommitment) {
                outSpentUtxos.push(sUtxo);
            }
        });
        const found = isFromUser || inSpentUtxos.length > 0 || outSpentUtxos.length > 0;
        if (found) {
            transactionHistory.push({
                ...trx,
                inSpentUtxos,
                outSpentUtxos,
            });
        }
    });
    return transactionHistory.sort((a, b) => b.blockTime - a.blockTime);
};
exports.getUserIndexTransactions = getUserIndexTransactions;
const findMatchingInstruction = (instructions, publicKeys) => {
    return instructions.find((instruction) => publicKeys.some((pubKey) => pubKey.equals(instruction.programId)));
};
exports.findMatchingInstruction = findMatchingInstruction;
/**
 * @async
 * @description This functions takes the indexer transaction event data and transaction,
 * including the signature, instruction parsed data, account keys, and transaction type.
 * @function enrichParsedTransactionEvents
 * @param {ParsedTransactionWithMeta} tx - The transaction object to process.
 * @param {IndexedTransaction[]} transactions - An array to which the processed transaction data will be pushed.
 * @returns {Promise<void>}
 */
async function enrichParsedTransactionEvents(event, transactions) {
    // check if transaction contains the meta data or not , else return without processing transaction
    const { tx, publicAmountSol, publicAmountSpl, relayerFee, firstLeafIndex, leaves, encryptedUtxos, message, } = event;
    if (!tx || !tx.meta || tx.meta.err)
        return;
    // check first whether we can find an instruction to a verifier program in the main instructions
    let instruction = (0, exports.findMatchingInstruction)(tx.transaction.message.instructions, constants_1.VERIFIER_PUBLIC_KEYS);
    // if we didn't find a main instruction to a verifier program we check the inner instructions
    // this is the case for private programs which call verifier two via cpi
    for (let innerInstruction of tx.meta.innerInstructions) {
        if (!instruction)
            instruction = (0, exports.findMatchingInstruction)(innerInstruction.instructions, constants_1.VERIFIER_PUBLIC_KEYS);
    }
    if (!instruction)
        return;
    const signature = tx.transaction.signatures[0];
    let accountKeys = instruction.accounts;
    let verifier = instruction.programId;
    const getTypeAndAmounts = (publicAmountSpl, publicAmountSol) => {
        let type = transaction_1.Action.SHIELD;
        let amountSpl = new anchor_1.BN(publicAmountSpl, 32, "be");
        let amountSol = new anchor_1.BN(publicAmountSol, 32, "be");
        let splIsU64 = amountSpl.lte(constants_1.MAX_U64);
        let solIsU64 = amountSol.lte(constants_1.MAX_U64);
        if (!splIsU64 || !solIsU64) {
            amountSpl = amountSpl.sub(constants_1.FIELD_SIZE).mod(constants_1.FIELD_SIZE).abs();
            amountSol = amountSol
                .sub(constants_1.FIELD_SIZE)
                .mod(constants_1.FIELD_SIZE)
                .abs()
                .sub(relayerFee);
            type =
                amountSpl.eq(constants_1.BN_0) && amountSol.eq(constants_1.BN_0)
                    ? transaction_1.Action.TRANSFER
                    : transaction_1.Action.UNSHIELD;
        }
        return { amountSpl, amountSol, type };
    };
    const { type, amountSpl, amountSol } = getTypeAndAmounts(publicAmountSpl, publicAmountSol);
    const convertToPublicKey = (key) => {
        return key instanceof web3_js_1.PublicKey ? key : new web3_js_1.PublicKey(key);
    };
    accountKeys = accountKeys.map((key) => convertToPublicKey(key));
    // 0: signingAddress
    // 1: systemProgram
    // 2: programMerkleTree
    // 3: transactionMerkleTree
    // 4: authority
    // 5: relayerRecipientSol
    // 6: senderSol
    // 7: recipientSol
    // 8: tokenProgram
    // 9: tokenAuthority
    // 10: senderSpl
    // 11: recipientSpl
    // 12: registeredVerifierPda
    // 13: logWrapper
    // 14: eventMerkleTree
    let relayerRecipientSol = accountKeys[5];
    let from = accountKeys[6];
    let to = accountKeys[7];
    let fromSpl = accountKeys[10];
    let toSpl = accountKeys[11];
    const nullifiers = event.nullifiers;
    let solTokenPoolIndex = type === transaction_1.Action.SHIELD ? 9 : 8;
    let changeSolAmount = new anchor_1.BN(tx.meta.postBalances[solTokenPoolIndex] -
        tx.meta.preBalances[solTokenPoolIndex]);
    changeSolAmount = changeSolAmount.lt(constants_1.BN_0)
        ? changeSolAmount.abs().sub(relayerFee)
        : changeSolAmount;
    transactions.push({
        blockTime: tx.blockTime * 1000,
        signer: accountKeys[0],
        signature,
        to,
        from,
        //TODO: check if this is the correct type after latest main?
        //@ts-ignore
        toSpl,
        fromSpl,
        verifier,
        relayerRecipientSol,
        type,
        changeSolAmount: changeSolAmount.toString("hex"),
        publicAmountSol: amountSol.toString("hex"),
        publicAmountSpl: amountSpl.toString("hex"),
        encryptedUtxos,
        leaves,
        nullifiers,
        relayerFee: relayerFee.toString("hex"),
        firstLeafIndex: firstLeafIndex.toString("hex"),
        message: Buffer.from(message),
    });
}
/**
 * @async
 * @description This functions takes the transactionMeta of  indexer events transactions and extracts relevant data from it
 * @function parseTransactionEvents
 * @param {(ParsedTransactionWithMeta | null)[]} indexerEventsTransactions - An array of indexer event transactions to process
 * @returns {Promise<void>}
 */
const parseTransactionEvents = (indexerEventsTransactions) => {
    const parsedTransactionEvents = [];
    indexerEventsTransactions.forEach((tx) => {
        if (!tx ||
            !tx.meta ||
            tx.meta.err ||
            !tx.meta.innerInstructions ||
            tx.meta.innerInstructions.length <= 0) {
            return;
        }
        tx.meta.innerInstructions.forEach((ix) => {
            ix.instructions.forEach((ixInner) => {
                if (!ixInner.data)
                    return;
                const data = bytes_1.bs58.decode(ixInner.data);
                const decodeData = new TransactionIndexerEvent().deserialize(data);
                if (decodeData) {
                    parsedTransactionEvents.push({
                        ...decodeData,
                        tx,
                    });
                }
            });
        });
    });
    return parsedTransactionEvents;
};
/**
 * @description Fetches transactions for the specified merkleTreeProgramId in batches
 * and process the incoming transaction using the enrichParsedTransactionEvents.
 * This function will handle retries and sleep to prevent rate-limiting issues.
 * @param {Connection} connection - The Connection object to interact with the Solana network.
 * @param {PublicKey} merkleTreeProgramId - The PublicKey of the Merkle tree program.
 * @param {ConfirmedSignaturesForAddress2Options} batchOptions - Options for fetching transaction batches,
 * including starting transaction signature (after), ending transaction signature (before), and batch size (limit).
 * @param {any[]} transactions - The array where the fetched transactions will be stored.
 * @returns {Promise<string>} - The signature of the last fetched transaction.
 */
// TODO: consider explicitly returning a new txs array instead of mutating the passed in one
async function getTransactionsBatch({ connection, merkleTreeProgramId, batchOptions, transactions, }) {
    const signatures = await connection.getConfirmedSignaturesForAddress2(new web3_js_1.PublicKey(merkleTreeProgramId), batchOptions, "confirmed");
    const lastSignature = signatures[signatures.length - 1];
    let txs = [];
    let index = 0;
    const signaturesPerRequest = 5;
    while (index < signatures.length) {
        try {
            const txsBatch = await connection.getParsedTransactions(signatures
                .slice(index, index + signaturesPerRequest)
                .map((sig) => sig.signature), {
                maxSupportedTransactionVersion: 0,
                commitment: "confirmed",
            });
            if (!txsBatch.some((t) => !t)) {
                txs = txs.concat(txsBatch);
                index += signaturesPerRequest;
            }
        }
        catch (e) {
            await (0, utils_1.sleep)(2000);
        }
    }
    const transactionEvents = txs.filter((tx) => {
        const accountKeys = tx.transaction.message.accountKeys;
        const splNoopIndex = accountKeys.findIndex((item) => {
            const itemStr = typeof item === "string" || item instanceof String
                ? item
                : item.pubkey.toBase58();
            return itemStr === new web3_js_1.PublicKey(spl_account_compression_1.SPL_NOOP_ADDRESS).toBase58();
        });
        if (splNoopIndex) {
            return txs;
        }
    });
    const parsedTransactionEvents = parseTransactionEvents(transactionEvents);
    parsedTransactionEvents.forEach((event) => {
        enrichParsedTransactionEvents(event, transactions);
    });
    return lastSignature;
}
/**
 * @description Fetches recent transactions for the specified merkleTreeProgramId.
 * This function will call getTransactionsBatch multiple times to fetch transactions in batches.
 * @param {Connection} connection - The Connection object to interact with the Solana network.
 * @param {ConfirmedSignaturesForAddress2Options} batchOptions - Options for fetching transaction batches,
 * including starting transaction signature (after), ending transaction signature (before), and batch size (limit).
 * @param {boolean} dedupe=false - Whether to deduplicate transactions or not.
 * @returns {Promise<indexedTransaction[]>} Array of indexedTransactions
 */
async function fetchRecentTransactions({ connection, batchOptions = {
    limit: 1,
    before: undefined,
    until: undefined,
}, transactions = [], }) {
    const batchSize = 1000;
    const rounds = Math.ceil(batchOptions.limit / batchSize);
    let batchBefore = batchOptions.before;
    for (let i = 0; i < rounds; i++) {
        const batchLimit = i === rounds - 1 ? batchOptions.limit - i * batchSize : batchSize;
        const lastSignature = await getTransactionsBatch({
            connection,
            merkleTreeProgramId: constants_1.merkleTreeProgramId,
            batchOptions: {
                limit: batchLimit,
                before: batchBefore,
                until: batchOptions.until,
            },
            transactions,
        });
        if (!lastSignature) {
            break;
        }
        batchBefore = lastSignature.signature;
        await (0, utils_1.sleep)(500);
    }
    return transactions.sort((a, b) => new anchor_1.BN(a.firstLeafIndex, "hex").toNumber() -
        new anchor_1.BN(b.firstLeafIndex, "hex").toNumber());
}
exports.fetchRecentTransactions = fetchRecentTransactions;
//# sourceMappingURL=fetchRecentTransactions.js.map