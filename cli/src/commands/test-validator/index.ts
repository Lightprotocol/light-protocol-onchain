import { Command, Flags } from "@oclif/core";
import { initTestEnv } from "../../utils/initTestEnv";
import { CustomLoader } from "../../utils/index";

class SetupCommand extends Command {
  static description = "Perform setup tasks";

  protected finally(_: Error | undefined): Promise<any> {
    process.exit();
  }

  static flags = {
    "skip-indexer": Flags.boolean({
      description: "Runs a test validator without starting a new indexer.",
      default: false,
    }),
    "skip-prover": Flags.boolean({
      description:
        "Runs a test validator without starting a new prover service.",
      default: false,
    }),
    "skip-forester": Flags.boolean({
      description:
        "Runs a test validator without starting a new forester service.",
      default: false,
    }),
    "skip-system-accounts": Flags.boolean({
      description:
        "Runs a test validator without initialized light system accounts.",
      default: false,
    }),
    "prove-compressed-accounts": Flags.boolean({
      description: "Enable proving of compressed accounts.",
      default: true,
      exclusive: ["skip-prover"],
    }),
    "prove-new-addresses": Flags.boolean({
      description: "Enable proving of new addresses.",
      default: true,
      exclusive: ["skip-prover"],
    }),
    "relax-indexer-version-constraint": Flags.boolean({
      description:
        "Disables indexer version check. Only use if you know what you are doing.",
      default: false,
      exclusive: ["skip-indexer"],
    }),
    "indexer-db-url": Flags.string({
      description:
        "Custom indexer database URL to store indexing data. By default we use an in-memory SQLite database.",
      required: false,
      exclusive: ["skip-indexer"],
    }),
    "limit-ledger-size": Flags.integer({
      description: "Keep this amount of shreds in root slots.",
      required: false,
      default: 10000,
    }),
  };

  async run() {
    const { flags } = await this.parse(SetupCommand);

    const loader = new CustomLoader("Performing setup tasks...\n");
    loader.start();
    await initTestEnv({
      checkPhotonVersion: !flags["relax-indexer-version-constraint"],
      forester: !flags["skip-forester"],
      indexer: !flags["skip-indexer"],
      limitLedgerSize: flags["limit-ledger-size"],
      photonDatabaseUrl: flags["indexer-db-url"],
      proveCompressedAccounts: flags["prove-compressed-accounts"],
      proveNewAddresses: flags["prove-new-addresses"],
      prover: !flags["skip-prover"],
      skipSystemAccounts: flags["skip-system-accounts"],
    });

    this.log("\nSetup tasks completed successfully \x1b[32m✔\x1b[0m");
  }
}

export default SetupCommand;
