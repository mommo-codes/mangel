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
  fields as wasmFields,
  languages as wasmLanguages,
  profiles as wasmProfiles,
  read as wasmRead,
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

/** A rule's refusal: an `Error` named `"Declined"`, whose message is a
 *  sentence a person can act on and whose `code` never changes. */
export interface Declined extends Error {
  name: "Declined";
  /** Why, as a stable code: `"no_unit"`, `"no_rule"`. See docs/decline-codes.md. */
  code: string;
}

/**
 * A rule's refusal. Every rule below throws one rather than guessing. A bad
 * market, field, profile or language code throws a plain `Error` instead.
 */
export function isDeclined(error: unknown): error is Declined {
  return error instanceof Error && error.name === "Declined";
}

/** Where a value comes from and where it goes. */
export interface ReadOptions {
  /** The market whose register the value goes into: `"se"`. */
  market: string;
  /** Who mangel reads for: see {@link profiles}. */
  profile: string;
  /** The languages printed on the pack, any number. */
  label?: string[];
  /** The language a language-neutral value is named in; the market's own
   *  by default. Must be an output language (see {@link languages}). */
  output?: string;
  /** The product's category, which a VAT rate is read against. */
  category?: string;
}

/** A value without a language or a market's spelling. */
export type Neutral =
  | { kind: "text"; text: string }
  | { kind: "size"; amount: string; unit: string }
  | { kind: "category"; category: Category }
  | { kind: "rate"; rate: number }
  | { kind: "deposit"; amount: number | null };

/** A field, read. */
export interface Read {
  /** The value as the market writes it: `"0,5L"` in Sweden. */
  value: string;
  /** The value without a language or a market's spelling. */
  neutral: Neutral;
  /** What was changed, as codes: `spacing`, `amount`, `unit_spelling`. */
  changes: string[];
}

/** A language mangel reads. */
export interface Language {
  code: string;
  name: string;
  /** Whether a value can be named in it, rather than only read. */
  output: boolean;
}

/**
 * Read `text` as `field` (see {@link fields}). Throws a {@link Declined}
 * when the value cannot be read, with `code` `"no_rule"` when the profile
 * has no rule for the field.
 */
export function read(field: string, text: string, options: ReadOptions): Read {
  assertReady();
  return wasmRead(field, text, options) as Read;
}

/** Every field {@link read} takes, by code. */
export function fields(): string[] {
  assertReady();
  return wasmFields() as string[];
}

/** Every profile {@link read} takes, by code. */
export function profiles(): string[] {
  assertReady();
  return wasmProfiles() as string[];
}

/** Every language mangel reads. */
export function languages(): Language[] {
  assertReady();
  return wasmLanguages() as Language[];
}

/** A size read into its amount and unit: `"56kg"` is `{ amount: "56", unit: "kg" }`.
 *  An amount that reads as two numbers is declined: `"1.000 g"` could be
 *  1 g or 1000 g. */
export function size(text: string, market: string): Size {
  assertReady();
  return wasmSize(text, market) as Size;
}

/** A cleaned name, checked: trimmed, and its first word starting with a
 *  capital where its script has capitals (Georgian, Arabic and Chinese do
 *  not). */
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
