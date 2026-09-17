/**
 * mangel — product data normalisation: irregular fields in, regular fields out.
 *
 * The same compiled Rust the Python package runs, vocabulary included. The
 * word lists are baked into the .wasm at build time, so the browser and the
 * backend cannot disagree about a word without also being on different
 * versions of the library. There is no TypeScript reimplementation of any
 * rule here, and there must never be one.
 */

import initWasm, { abbreviations as wasmAbbreviations } from "../wasm/mangel_wasm.js";

let ready = false;

/**
 * Load the WebAssembly module. Call once, at application start.
 *
 * Every other function throws until this resolves. That is deliberate: a
 * normaliser that quietly handed input back unchanged while warming up would
 * look exactly like one that had nothing to change.
 */
export async function init(module?: Parameters<typeof initWasm>[0]): Promise<void> {
  await initWasm(module);
  ready = true;
}

/** Whether {@link init} has completed. */
export function isReady(): boolean {
  return ready;
}

function assertReady(): void {
  if (!ready) {
    throw new Error(
      "mangel: call `await init()` once before using the library. " +
        "Refusing to answer rather than returning a wrong answer.",
    );
  }
}

/**
 * A market's abbreviations, as a new `Map`: each word as it is written in
 * product data, mapped to the abbreviation the golden standard uses. Entries
 * are in sorted order.
 *
 * Throws for a market code mangel has no conventions for. Codes are exact:
 * `"se"`, not `"SE"`. There is no default market.
 */
export function abbreviations(market: string): ReadonlyMap<string, string> {
  assertReady();
  return wasmAbbreviations(market) as Map<string, string>;
}
