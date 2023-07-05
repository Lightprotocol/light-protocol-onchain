const snarkjs = require("snarkjs");
const { unstringifyBigInts, stringifyBigInts, leInt2Buff } =
  require("ffjavascript").utils;

import { Idl } from "@coral-xyz/anchor";

import { VerifierError, VerifierErrorCode } from "@lightprotocol/zk.js";

import {
  VerifierIdls,
  ProofInputs,
  ParsedPublicInputs,
  CircuitNames,
} from "./generics";

export type proofData = {
  pi_a: string[];
  pi_b: string[][];
  pi_c: string[];
  protocol: string;
  curve: string;
};

export type vKeyData = {
  protocol: string;
  curve: string;
  nPublic: number;
  vk_alpha_1: string[];
  vk_beta_2: string[][];
  vk_gamma_2: string[][];
  vk_delta_2: string[][];
  vk_alphabeta_12: ArrayConstructor[][][];
  IC: string[][];
};

export class Prover<
  VerifierIdl extends VerifierIdls,
  CircuitName extends CircuitNames,
> {
  public circuitName!: CircuitName;
  public idl: Idl;
  public firstPath: string;
  public wasmPath!: string;
  public zkeyPath!: string;
  public proofInputs!: ProofInputs<VerifierIdl, CircuitName>;
  public publicInputs: string[] = [];
  public vKey!: vKeyData;
  public proof!: proofData;

  constructor(idl: Idl, firstPath: string) {
    this.idl = idl;
    this.firstPath = firstPath;
  }

  async addProofInputs(proofInputs: any) {
    // Filter accounts that contain zK and either PublicInputs or ProofInputs

    const ZKAccountNames = this.idl.accounts
      ?.filter((account) =>
        /zK.*(?:PublicInputs|ProofInputs)|zk.*(?:PublicInputs|ProofInputs)/.test(
          account.name,
        ),
      )
      .map((account) => account.name);

    // Extract the circuit names and store them in a Set to get unique names
    const circuitNameRegex =
      /zK(.*?)ProofInputs|zK(.*?)PublicInputs|zk(.*?)ProofInputs|zk(.*?)PublicInputs/;
    const uniqueCircuitNames = new Set<string>();

    ZKAccountNames?.forEach((name) => {
      const match = name.match(circuitNameRegex);
      if (match) {
        uniqueCircuitNames.add(match[1] || match[2] || match[3] || match[4]);
      }
    });

    this.circuitName = Array.from(uniqueCircuitNames)[0] as CircuitName;

    // After Retrieving circuitName ==> build wasm and zkey paths for the circuit
    this.wasmPath =
      this.firstPath + `/${this.circuitName}_js/${this.circuitName}.wasm`;
    this.zkeyPath = this.firstPath + `/${this.circuitName}.zkey`;

    const circuitIdlObject = this.idl.accounts!.find(
      (account) =>
        account.name.toUpperCase() ===
        `zK${this.circuitName}ProofInputs`.toUpperCase(),
    );

    if (!circuitIdlObject) {
      throw new Error(
        `${`zK${this.circuitName}ProofInputs`} does not exist in anchor idl`,
      );
    }

    const fieldNames = circuitIdlObject.type.fields.map(
      (field: { name: string }) => field.name,
    );
    const inputKeys: string[] = [];

    fieldNames.forEach((fieldName: string) => {
      inputKeys.push(fieldName);
    });

    let inputsObject: { [key: string]: any } = {};

    inputKeys.forEach((key) => {
      inputsObject[key] = proofInputs[key];
      if (!inputsObject[key])
        throw new Error(
          `Missing input --> ${key.toString()} in circuit ==> ${
            this.circuitName
          }`,
        );
    });
    this.proofInputs = inputsObject as ProofInputs<VerifierIdl, CircuitName>;
  }

  async fullProve() {
    const { proof, publicSignals } = await snarkjs.groth16.fullProve(
      stringifyBigInts(this.proofInputs),
      this.wasmPath,
      this.zkeyPath,
    );

    this.publicInputs = publicSignals;
    this.proof = proof;
  }

  async getVkey() {
    const vKey = await snarkjs.zKey.exportVerificationKey(this.zkeyPath);
    this.vKey = vKey;
  }

  async verify(proof?: proofData): Promise<boolean> {
    // verifies the proof generated by this class or a passed in proof
    await this.getVkey();
    const res = await snarkjs.groth16.verify(
      this.vKey,
      this.publicInputs,
      this.proof,
    );
    return res;
  }

  parseProofToBytesArray(proof: proofData): {
    proofA: number[];
    proofB: number[][];
    proofC: number[];
  } {
    let proofJson = JSON.stringify(proof, null, 1);
    var mydata = JSON.parse(proofJson.toString());
    try {
      for (var i in mydata) {
        if (i == "pi_a" || i == "pi_c") {
          for (var j in mydata[i]) {
            mydata[i][j] = Array.from(
              leInt2Buff(unstringifyBigInts(mydata[i][j]), 32),
            ).reverse();
          }
        } else if (i == "pi_b") {
          for (var j in mydata[i]) {
            for (var z in mydata[i][j]) {
              mydata[i][j][z] = Array.from(
                leInt2Buff(unstringifyBigInts(mydata[i][j][z]), 32),
              );
            }
          }
        }
      }

      return {
        proofA: [mydata.pi_a[0], mydata.pi_a[1]].flat(),
        proofB: [
          mydata.pi_b[0].flat().reverse(),
          mydata.pi_b[1].flat().reverse(),
        ].flat(),
        proofC: [mydata.pi_c[0], mydata.pi_c[1]].flat(),
      };
    } catch (error) {
      console.error("error while parsing the proof!");
      throw error;
    }
  }

  // mainly used to parse the public signals of groth16 fullProve
  parseToBytesArray(publicSignals: string[]): number[][] {
    const publicInputsJson = JSON.stringify(publicSignals, null, 1);
    var publicInputsBytesJson = JSON.parse(publicInputsJson.toString());
    try {
      var publicInputsBytes = new Array<Array<number>>();
      for (var i in publicInputsBytesJson) {
        let ref: Array<number> = Array.from([
          ...leInt2Buff(unstringifyBigInts(publicInputsBytesJson[i]), 32),
        ]).reverse();
        publicInputsBytes.push(ref);
      }

      return publicInputsBytes;
    } catch (error) {
      console.error("error while parsing public inputs!");
      throw error;
    }
  }

  async fullProveAndParse() {
    await this.fullProve();

    const parsedPublicInputsObj = this.parseToBytesArray(this.publicInputs);
    const parsedProofObj = this.parseProofToBytesArray(this.proof);

    return {
      parsedProof: parsedProofObj,
      parsedPublicInputs: parsedPublicInputsObj,
    };
  }

  parsePublicInputsFromArray(
    publicInputsBytes: number[][],
  ): ParsedPublicInputs<VerifierIdl, CircuitName> {
    type SizeObject = {
      [key: string]: number[];
    };

    function getNrPublicInputs(input: SizeObject): number {
      let arr = [];
      for (const key in input) {
        arr.push(...input[key]);
      }
      const updatedArray = arr.map((value) => (value === 0 ? 1 : value));

      return updatedArray.reduce(
        (accumulator, currentValue) => accumulator + currentValue,
        0,
      );
    }

    function getSize(type: any): number[] {
      const sizeArray = [];
      if (typeof type === "string") {
        sizeArray.push(0);
      } else if (Array.isArray(type)) {
        if (typeof type[0] === "string") {
          sizeArray.push(type[1]);
        } else {
          sizeArray.push(...getSize(type[0]), type[1]);
        }
      } else {
        return getSize(type.array);
      }

      return sizeArray;
    }

    function getObjectSizes(idlObject: any): SizeObject {
      let output: SizeObject = {};

      for (const field of idlObject[0].type.fields) {
        output[field.name] = getSize(field.type);
      }

      return output;
    }

    type NestedNumberArray = number | NestedNumberArray[];

    type OutputObject = {
      [key: string]: NestedNumberArray;
    };

    function spreadArrayToObject(input: number[][], sizes: SizeObject): any {
      let currentIndex = 0;

      const output: OutputObject = {};

      for (const key in sizes) {
        if (!sizes.hasOwnProperty(key)) {
          continue;
        }

        const shape = sizes[key];
        if (shape.length === 1 && shape[0] === 0) {
          output[key] = input[currentIndex];
          currentIndex += 1;
        } else {
          const totalElements = shape.reduce(
            (accumulator, size) => accumulator * size,
            1,
          );
          const slicedArray = input.slice(
            currentIndex,
            currentIndex + totalElements,
          );
          let reshapedArray: any = slicedArray;

          if (shape.length > 1) {
            let currentData = reshapedArray;
            for (let i = 0; i < shape.length - 1; i++) {
              currentData = currentData.reduce(
                (accumulator: any[], _: any, index: number) => {
                  if (index % shape[i] === 0) {
                    accumulator.push(
                      currentData.slice(index, index + shape[i]),
                    );
                  }
                  return accumulator;
                },
                [],
              );
            }
            reshapedArray = currentData;
          }

          output[key] = reshapedArray;
          currentIndex += totalElements;
        }
      }

      if (currentIndex !== input.length) {
        throw new Error(
          `Input array length mismatch: ${currentIndex} != ${input.length}`,
        );
      }

      return output;
    }

    if (!publicInputsBytes) {
      throw new VerifierError(
        VerifierErrorCode.PUBLIC_INPUTS_UNDEFINED,
        "parsePublicInputsFromArray",
        this.circuitName,
      );
    }

    const publicInputs_IdlObject = this.idl.accounts!.find(
      (account) =>
        account.name.toUpperCase() ===
        `ZK${this.circuitName}PublicInputs`.toUpperCase(),
    );

    const key_sizes = getObjectSizes([publicInputs_IdlObject]);
    const nrPublicInputs = getNrPublicInputs(key_sizes);

    if (publicInputsBytes.length != nrPublicInputs) {
      throw new VerifierError(
        VerifierErrorCode.INVALID_INPUTS_NUMBER,
        "parsePublicInputsFromArray",
        `${this.circuitName}: publicInputsBytes.length invalid ${publicInputsBytes.length} != ${nrPublicInputs}`,
      );
    }

    const result = spreadArrayToObject(publicInputsBytes, key_sizes);

    return result as ParsedPublicInputs<VerifierIdl, CircuitName>;
  }
}
