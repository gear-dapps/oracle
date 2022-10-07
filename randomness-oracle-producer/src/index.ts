import {
  GearApi,
  GearKeyring,
  getWasmMetadata,
  CreateType,
  decodeAddress,
  Hex,
} from "@gear-js/api";
import * as dotenv from "dotenv";
import { readFileSync } from "fs";
import { Random } from "./types";
import { fetchRandomValue } from "./utils";

dotenv.config();

const ENDPOINT_URL = process.env.ENDPOINT_URL || "";

const ORACLE_ADDRESS: Hex = (process.env.ORACLE_ADDRESS as Hex) || "0x";
const ORACLE_META_WASM_PATH = process.env.ORACLE_META_WASM_PATH || "";
const ORACLE_META_WASM_BUFFER = readFileSync(ORACLE_META_WASM_PATH);

const KEYRING_PATH = process.env.KEYRING_PATH || "";
const KEYRING_PASSPHRASE = process.env.KEYRING_PASSPHRASE || "";
const KEYRING = GearKeyring.fromJson(
  readFileSync(KEYRING_PATH).toString(),
  KEYRING_PASSPHRASE
);

const updateOracleValue = async (data: [number, Random], gearApi: GearApi) => {
  const [round, random] = data;

  try {
    const oracleMeta = await getWasmMetadata(ORACLE_META_WASM_BUFFER);

    const payload = CreateType.create(
      "Action",
      {
        SetRandomValue: {
          round,
          value: {
            randomness: [random.randomness[0], random.randomness[1]],
            signature: random.signature,
            prev_signature: random.prevSignature,
          },
        },
      },
      oracleMeta
    );

    const gas = await gearApi.program.calculateGas.handle(
      decodeAddress(KEYRING.address),
      ORACLE_ADDRESS,
      payload.toHex(),
      0,
      true,
      oracleMeta
    );

    let extrinsic = gearApi.message.send(
      {
        destination: ORACLE_ADDRESS,
        payload: payload.toHex(),
        gasLimit: gas.min_limit,
        value: 0,
      },
      undefined,
      "String"
    );

    await extrinsic.signAndSend(KEYRING, (event: any) => {
      if (event.isError) {
        throw new Error("Can't send tx");
      } else {
        console.log(`[+] UpdateValue(${round}, ${random})`);
      }
    });
  } catch (error: any) {
    console.log(`[-] Failed to send tx: ${error}`);
  }
};

const main = async () => {
  // 1. Connect to node
  const gearApi = await GearApi.create({
    providerAddress: ENDPOINT_URL,
  });

  console.log(
    `[+] Started with: ${await gearApi.nodeName()}-${await gearApi.nodeVersion()}`
  );

  // 2. Feed oracle via external API
  setInterval(async () => {
    const data = await fetchRandomValue();
    console.log(`New tick: ${data[0]}`);

    await updateOracleValue(data, gearApi);
  }, 30000);
};

main();
