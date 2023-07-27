import { Args, Command } from "@oclif/core";
import { toSnakeCase } from "../../psp-utils/buildPSP";
import { start_test_validator } from "../../utils";
import { executeCommand } from "../../psp-utils";

export default class TestCommand extends Command {
  static description = "Deploys your PSP on a local testnet and runs test";

  static args = {
    name: Args.string({
      name: "NAME",
      description: "The name of the project",
      required: true,
    }),
    address: Args.string({
      name: "NAME",
      description: "The name of the project",
      required: true,
    }),
  };

  async run() {
    const { args } = await this.parse(TestCommand);
    let { name, address } = args;

    const programName = toSnakeCase(name!);
    const path = `./target/deploy/${programName}.so`;
    await start_test_validator({
      additonalPrograms: [{ address: address!, path }],
    });

    await executeCommand({
      command: `yarn`,
      args: [`ts-mocha`, `-t`, `2000000`, `tests/${name}.ts`, `--exit`],
    });
    this.exit(0);
  }
}
