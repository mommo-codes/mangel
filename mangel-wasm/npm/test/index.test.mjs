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
