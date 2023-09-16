export * from "./createAccounts";
export * from "./testChecks";
export * from "./setUpMerkleTree";
export * from "./initLookUpTable";
export * from "./constants_market_place";
export * from "./functionalCircuit";
export * from "./constants_system_verifier";
export * from "./updateMerkleTree";
export * from "./testRelayer";
export * from "./userTestAssertHelper";
export * from "./testTransaction";
export * from "./airdrop";
export function generateRandomTestAmount(min = 0.2, max = 2, decimals) {
    const randomAmount = Math.random() * (max - min) + min;
    return +randomAmount.toFixed(decimals);
}
//# sourceMappingURL=index.js.map