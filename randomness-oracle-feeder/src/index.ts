import {
  GearApi,
  GearKeyring,
  getWasmMetadata,
  CreateType,
  decodeAddress,
  Hex,
  Metadata,
} from "@gear-js/api";
import { KeyringPair } from "@polkadot/keyring/types";
import * as dotenv from "dotenv";
import { readFileSync } from "fs";
import { Random } from "./types";
import { fetchRandomValue } from "./utils";

dotenv.config();

const ENDPOINT_URL = process.env.ENDPOINT_URL || "";

const ORACLE_ADDRESS: Hex = (process.env.ORACLE_ADDRESS as Hex) || "0x";
const ORACLE_META_WASM_PATH = process.env.ORACLE_META_WASM_PATH || "";
const ORACLE_META_WASM_BUFFER = readFileSync(ORACLE_META_WASM_PATH);

const KEYRING_PASSPHRASE = process.env.KEYRING_PASSPHRASE;
const KEYRING_PATH = process.env.KEYRING_PATH;
const KEYRING_MNEMONIC = process.env.KEYRING_MNEMONIC;
const KEYRING_SEED = process.env.KEYRING_SEED;

const getKeyring = async (): Promise<KeyringPair | undefined> => {
  if (KEYRING_MNEMONIC !== undefined) {
    return await GearKeyring.fromMnemonic(KEYRING_MNEMONIC);
  }

  if (KEYRING_SEED !== undefined) {
    return await GearKeyring.fromSeed(KEYRING_SEED);
  }

  if (KEYRING_PATH !== undefined && KEYRING_PASSPHRASE !== undefined) {
    return GearKeyring.fromJson(
      readFileSync(KEYRING_PATH).toString(),
      KEYRING_PASSPHRASE
    );
  }

  return undefined;
};

const updateOracleValue = async (
  gearApi: GearApi,
  oracleMeta: Metadata,
  keyring: KeyringPair,
  data: [number, Random]
) => {
  const [round, random] = data;

  try {
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
      decodeAddress(keyring.address),
      ORACLE_ADDRESS,
      payload.toHex(),
      0,
      true,
      oracleMeta
    );

    let extrinsic = gearApi.message.send({
      destination: ORACLE_ADDRESS,
      payload: payload.toHex(),
      gasLimit: gas.min_limit,
      value: 0,
    });

    await extrinsic.signAndSend(keyring, (event: any) => {
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

  // 2. Load oracle wasm metadata
  const oracleMeta = await getWasmMetadata(ORACLE_META_WASM_BUFFER);

  // 3. Load Keyring from one of provided methods
  const keyring = await getKeyring();
  if (keyring === undefined) {
    console.log("[-] Unable to load keypair by provided methods");
    return;
  }

  // 4. Feed oracle via external API
  setInterval(async () => {
    const data = await fetchRandomValue();
    console.log(`New tick: ${data[0]}`);

    await updateOracleValue(gearApi, oracleMeta, keyring, data);
  }, 30000);
};

main();
