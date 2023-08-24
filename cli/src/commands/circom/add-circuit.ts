import { Args, Command } from "@oclif/core";
import { addCircuit } from "../psp/add-circuit";

export const PSP_DEFAULT_PROGRAM_ID =
  "Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS";

export default class InitCommand extends Command {
  static description =
    "Add a circom circuit to your anchor circom or PSP project.";

  static args = {
    name: Args.string({
      name: "NAME",
      description: "The name of the circuit",
      required: true,
    }),
  };

  async run() {
    const { args } = await this.parse(InitCommand);
    let { name } = args;

    this.log("🚀 Adding a circuit...");
    await addCircuit({ name, circom: true });
    this.log("✅ Project initialized successfully");
  }
}
