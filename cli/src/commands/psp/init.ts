import { Args, Command } from "@oclif/core";
import { snakeCase } from "snake-case";
import { downloadCargoGenerateIfNotExists } from "../../psp-utils/download";
import { executeCommandInDir } from "../../psp-utils/process";
import { executeCargoGenerate } from "../../psp-utils/toolchain";
import * as path from "path";
import { PSP_TEMPLATE_TAG } from "../../psp-utils/constants";
import { camelToScreamingSnake } from "../../utils";
import { renameFolder } from "../../psp-utils/utils";
import { toCamelCase } from "../../psp-utils";

export enum ProjectType {
  PSP = "psp",
  CIRCOM = "circom",
}

export const PSP_DEFAULT_PROGRAM_ID =
  "Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS";

export default class InitCommand extends Command {
  static description = "Initialize a PSP project.";

  static args = {
    name: Args.string({
      name: "NAME",
      description: "The name of the project",
      required: true,
    }),
  };

  async run() {
    const { args } = await this.parse(InitCommand);
    let { name } = args;

    this.log("🚀 Initializing PSP project...");
    await initRepo(name, ProjectType.PSP);

    this.log("✅ Project initialized successfully");
  }
}

export const initRepo = async (name: string, type: ProjectType) => {
  var circomName = snakeCase(name);
  var rustName = snakeCase(name);

  await executeCargoGenerate({
    args: [
      "generate",
      // "--git",
      // "https://github.com/Lightprotocol/psp-template",
      // --tag,
      // PSP_TEMPLATE_TAG,
      "--path",
      "/home/ananas/test_light/psp-template",
      "psp-template",
      "--name",
      name,
      "--define",
      `circom-name=${circomName}`,
      "--define",
      `rust-name=${rustName}`,
      "--define",
      `program-id=${PSP_DEFAULT_PROGRAM_ID}`,
      "--define",
      `VERIFYING_KEY_NAME=${camelToScreamingSnake(circomName)}`,
      "--define",
      `type=${type}`,
      "--define",
      `circom-name-camel-case=${toCamelCase(circomName)}`,
    ],
  });

  await renameFolder(
    `${process.cwd()}/${name}/circuits/circuit_${type}`,
    `${process.cwd()}/${name}/circuits/${name}`
  );
  await renameFolder(
    `${process.cwd()}/${name}/tests_${type}`,
    `${process.cwd()}/${name}/tests`
  );
  await renameFolder(
    `${process.cwd()}/${name}/programs_${type}`,
    `${process.cwd()}/${name}/programs`
  );

  await executeCommandInDir("yarn", ["install"], name);
};
