import fetch from "isomorphic-unfetch";
import { Random } from "./types";

export const fetchRandomValue = async (): Promise<[number, Random]> => {
  const rawData = await (
    await fetch("https://api.drand.sh/public/latest")
  ).json();

  const randomness: Uint8Array = Uint8Array.from(
    Buffer.from(rawData.randomness, "hex")
  );

  return [
    rawData.round,
    {
      randomness: [randomness.slice(0, 16), randomness.slice(16)],
      signature: rawData.signature,
      prevSignature: rawData.previous_signature,
    },
  ];
};
