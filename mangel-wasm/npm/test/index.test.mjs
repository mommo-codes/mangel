/**
 * Runtime tests for the TypeScript/wasm build.
 *
 * What the vocabulary says is tested once, in Rust. What is tested here is
 * that the compiled tables survive the wasm boundary, that a bad market is
 * refused rather than defaulted, and that the wrapper refuses to work before
 * it is ready rather than answering wrongly.
 */

import { test } from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";

import * as mangel from "../dist/index.js";

// `--target web` expects to fetch the .wasm; under Node we hand it the bytes.
const wasmBytes = await readFile(new URL("../wasm/mangel_wasm_bg.wasm", import.meta.url));

test("refuses to answer before init", () => {
  assert.equal(mangel.isReady(), false);
  assert.throws(() => mangel.abbreviations("se"), /call `await init\(\)`/);
});

test("init makes it ready", async () => {
  await mangel.init({ module_or_path: wasmBytes });
  assert.equal(mangel.isReady(), true);
});

test("the compiled table crosses the boundary", () => {
  // "Laktosfri" -> "LF" is the entry the vocabulary was scaffolded with. It
  // stands in for the whole table: if it arrives, the table did.
  const table = mangel.abbreviations("se");
  assert.ok(table instanceof Map);
  assert.equal(table.get("Laktosfri"), "LF");
});

test("the core order survives", () => {
  const words = [...mangel.abbreviations("se").keys()];
  const bytes = (word) => Buffer.from(word, "utf8");
  assert.deepEqual(words, [...words].sort((a, b) => Buffer.compare(bytes(a), bytes(b))));
});

test("each call gets its own copy", () => {
  mangel.abbreviations("se").set("Laktosfri", "changed");
  assert.equal(mangel.abbreviations("se").get("Laktosfri"), "LF");
});

test("anything but an exact market code is refused", () => {
  for (const code of ["SE", " se", "se ", "", "no", "sweden"]) {
    assert.throws(() => mangel.abbreviations(code), /unknown market/);
  }
});

test("the refusal says what would have worked", () => {
  assert.throws(() => mangel.abbreviations("SE"), /"se"/);
});

test("the sheet's rules cross the boundary", () => {
  assert.deepEqual({ ...mangel.size("56kg", "se") }, { amount: "56", unit: "kg" });
  assert.equal(mangel.cleanedName(" Mellanmjölk "), "Mellanmjölk");
  assert.deepEqual({ ...mangel.category("pizza", "se") }, { name: "Fryst Pizza", group: "Fryst", vat: 12 });
  assert.equal(mangel.category("Påsk", "se").vat, null);
  assert.equal(mangel.categories("se").length, 180);
  assert.equal(mangel.vat("Frukt", "", "se"), 12);
  assert.equal(mangel.deposit("", "se"), undefined);
  assert.equal(mangel.deposit("3 kr", "se"), 3);
});

test("a decline is thrown with its reason, and is told apart from a bad market", () => {
  let caught;
  try {
    mangel.vat("Frukt", "25", "se");
  } catch (error) {
    caught = error;
  }
  assert.ok(mangel.isDeclined(caught));
  assert.match(caught.message, /does not match Frukt/);
  assert.throws(() => mangel.size("56kg", "SE"), (error) => !mangel.isDeclined(error));
});

test("read crosses the boundary with its value, neutral value and changes", () => {
  const read = mangel.read("size", "0,5 l", { market: "se", profile: "name_scrubbing" });
  assert.equal(read.value, "0,5L");
  assert.deepEqual({ ...read.neutral }, { kind: "size", amount: "0.5", unit: "l" });
  assert.deepEqual([...read.changes], ["spacing", "unit_spelling"]);
  const vat = mangel.read("vat", "", { market: "se", profile: "name_scrubbing", category: "Frukt" });
  assert.deepEqual({ ...vat.neutral }, { kind: "rate", rate: 12 });
});

test("every decline carries its code, and a bad code is not a decline", () => {
  const codeOf = (run) => {
    try {
      run();
    } catch (error) {
      return mangel.isDeclined(error) ? error.code : `not declined: ${error.message}`;
    }
    return "read";
  };
  assert.equal(codeOf(() => mangel.read("size", "56", { market: "se", profile: "name_scrubbing" })), "no_unit");
  assert.equal(codeOf(() => mangel.read("weight", "500 g", { market: "se", profile: "laundry_room" })), "no_rule");
  assert.equal(codeOf(() => mangel.size("1.000 g", "se")), "two_readings");
  assert.match(codeOf(() => mangel.read("size", "5 g", { market: "se", profile: "nope" })), /^not declined: unknown profile/);
  assert.match(
    codeOf(() => mangel.read("name", "Mjölk", { market: "se", profile: "name_scrubbing", output: "tr" })),
    /^not declined: "tr" is read but not written/,
  );
});

test("the lists come from the core", () => {
  assert.ok(mangel.fields().includes("weight"));
  assert.deepEqual(mangel.profiles(), ["name_scrubbing", "laundry_room"]);
  const output = mangel.languages().filter((language) => language.output).map((language) => language.code);
  assert.deepEqual(output, ["sv", "da", "nb", "hu", "hr", "de", "el", "ka", "en"]);
});
