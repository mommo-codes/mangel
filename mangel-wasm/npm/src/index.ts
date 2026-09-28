/**
 * mangel — product data normalisation: irregular fields in, regular fields out.
 *
 * The same compiled Rust the Python package runs, vocabulary included. The
 * word lists are baked into the .wasm at build time, so the browser and the
 * backend cannot disagree about a word without also being on different
 * versions of the library. There is no TypeScript reimplementation of any
 * rule here, and there must never be one.
 */

import initWasm, {
  abbreviations as wasmAbbreviations,
  categories as wasmCategories,
  category as wasmCategory,
  cleanedName as wasmCleanedName,
  deposit as wasmDeposit,
  size as wasmSize,
  vat as wasmVat,
} from "../wasm/mangel_wasm.js";

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

/** A size, read: the amount as a decimal string, the unit in the golden
 *  standard's spelling. */
export interface Size {
  amount: string;
  unit: string;
}

/** A category of the market's list. `vat` is `null` for a category whose
 *  products carry more than one rate, so the VAT is typed per product. */
export interface Category {
  name: string;
  group: string;
  vat: number | null;
}

/**
 * A rule's refusal. Every rule below throws one — an `Error` named
 * `"Declined"` whose message is a sentence a person can act on — rather
 * than guessing. A bad market code throws a plain `Error` instead.
 */
export function isDeclined(error: unknown): error is Error {
  return error instanceof Error && error.name === "Declined";
}

/** A size read into its amount and unit: `"56kg"` is `{ amount: "56", unit: "kg" }`. */
export function size(text: string, market: string): Size {
  assertReady();
  return wasmSize(text, market) as Size;
}

/** A cleaned name, checked: trimmed, and its first word starting with a capital. */
export function cleanedName(text: string): string {
  assertReady();
  return wasmCleanedName(text);
}

/** The category meant by what was typed: its name, a group of one, or a keyword. */
export function category(text: string, market: string): Category {
  assertReady();
  return wasmCategory(text, market) as Category;
}

/** Every category of the market, sorted by name. */
export function categories(market: string): Category[] {
  assertReady();
  return wasmCategories(market) as Category[];
}

/** The VAT for a product in `category`, given what was typed in its field. */
export function vat(category: string, text: string, market: string): number {
  assertReady();
  return wasmVat(category, text, market);
}

/** A deposit, or `undefined` for an empty field. */
export function deposit(text: string, market: string): number | undefined {
  assertReady();
  return wasmDeposit(text, market);
}
