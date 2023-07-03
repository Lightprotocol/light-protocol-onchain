import { Command, Flags, Args } from "@oclif/core";
import { PublicKey } from "@solana/web3.js";
import {
  CustomLoader,
  generateSolanaTransactionURL,
  getUser,
} from "../../utils/utils";
import { ADMIN_AUTH_KEYPAIR } from "@lightprotocol/zk.js";

class UnshieldCommand extends Command {
  static description = "Unshield SPL tokens for a user";
  static usage = "unshield:spl <AMOUNT> <TOKEN> <RECIPIENT_ADDRESS> [FLAGS]";
  static examples = [
    "$ light unshield:spl 15 USDC <RECIPIENT_ADDRESS>",
  ];

  static flags = {
    "minimum-lamports": Flags.boolean({
      char: "m",
      description:
        "Whether to use the minimum required lamports for the unshield transaction",
      default: false,
    }),
  };

  static args = {
    amount: Args.string({
      name: "AMOUNT",
      description: "The SPL amount to unshield",
      required: true,
    }),
    token: Args.string({
      name: "TOKEN",
      description: "The SPL token to unshield",
      parse: async (token) => token.toUpperCase(), 
      required: true,
    }),
    recipient_address: Args.string({
      name: "RECIPIENT_ADDRESS",
      description: "The SPL account address of recipient.",
      required: true,
    }),
  };

  async run() {

    const { args, flags } = await this.parse(UnshieldCommand);
    const amountSpl = args.amount;
    const token = args.token;
    const recipient = args.recipient_address;
    const minimumLamports = flags["minimum-lamports"];

    const loader = new CustomLoader("Performing token unshield...\n");
    loader.start();

    try {

      const user = await getUser();
      const response = await user.unshield({
        token,
        recipient: new PublicKey(recipient),
        publicAmountSpl: amountSpl,
        minimumLamports,
      });
      this.log(generateSolanaTransactionURL("tx", `${response.txHash.signatures?.slice(-1)}`, "custom"));
      this.log(
        `\nSuccessfully unshielded ${amountSpl} ${token}`,
        "\x1b[32m✔\x1b[0m"
      );
      loader.stop();
    } catch (error) {
      this.error(`Failed to unshield ${token}!\n${error}`);
    }
  }
}

export default UnshieldCommand;
