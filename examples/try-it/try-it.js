// The try-it page. Every answer comes from the compiled library through
// `read`, and every list on the page (fields, profiles, languages) comes from
// the library too. This file routes a line to `read` and prints what comes
// back. No rule is written here, and none may be.

import * as mangel from "../../mangel-wasm/npm/dist/index.js";

// mangel has conventions for one market so far.
const MARKET = "se";

// What each change code means, for the reader.
const CHANGES = {
  spacing: "spacing",
  amount: "amount written the market's way",
  unit_spelling: "unit spelled the market's way",
};

const $ = (id) => document.getElementById(id);

function option(select, value, label, selected) {
  const element = document.createElement("option");
  element.value = value;
  element.textContent = label;
  element.selected = value === selected;
  select.append(element);
}

function describe(neutral) {
  switch (neutral.kind) {
    case "size":
      return `amount ${neutral.amount}, unit ${neutral.unit}`;
    case "category":
      return `category ${neutral.category.name}, group ${neutral.category.group}`;
    case "rate":
      return `rate ${neutral.rate}%`;
    case "deposit":
      return neutral.amount === null ? "no deposit" : `deposit ${neutral.amount}`;
    default:
      return "";
  }
}

function row(input, kind, tag, value, notes) {
  const tr = document.createElement("tr");
  const left = document.createElement("td");
  left.className = "input";
  left.textContent = input === "" ? "(empty)" : JSON.stringify(input);
  const right = document.createElement("td");
  right.className = kind;
  const badge = document.createElement("span");
  badge.className = "tag";
  badge.textContent = tag;
  const shown = document.createElement("span");
  shown.className = kind === "ok" ? "value" : "";
  shown.textContent = value;
  right.append(badge, shown);
  for (const note of notes) {
    const line = document.createElement("div");
    line.className = "note";
    if (Array.isArray(note)) {
      line.append("changed: ");
      for (const code of note) {
        const chip = document.createElement("span");
        chip.className = "tag change";
        chip.textContent = CHANGES[code] ?? code;
        line.append(chip);
      }
    } else {
      line.textContent = note;
    }
    right.append(line);
  }
  tr.append(left, right);
  return tr;
}

function chosenLabel() {
  return [...document.querySelectorAll("#label input:checked")].map((box) => box.value);
}

function render() {
  const field = $("field").value;
  const profile = $("profile").value;
  const output = $("output").value;
  const label = chosenLabel();
  $("category-label").hidden = field !== "vat";
  $("rules-note").textContent =
    `Market ${MARKET}. No rule reads the label or output language yet, so the answers are ` +
    "the same whatever you pick. They start to matter with weight and volume (#18).";

  const body = $("results");
  body.replaceChildren();
  for (const line of $("values").value.split("\n")) {
    try {
      const read = mangel.read(field, line, {
        market: MARKET,
        profile,
        label,
        output,
        category: $("category").value,
      });
      const notes = [];
      if (read.changes.length) notes.push(read.changes);
      else notes.push("nothing changed: already written the market's way");
      const neutral = describe(read.neutral);
      if (neutral) notes.push(neutral);
      body.append(row(line, "ok", "read", read.value, notes));
    } catch (error) {
      if (mangel.isDeclined(error) && error.code === "no_rule") {
        body.append(row(line, "none", "no rule", error.message, [`code ${error.code}`]));
      } else if (mangel.isDeclined(error)) {
        body.append(row(line, "no", "declined", error.message, [`code ${error.code}`]));
      } else {
        body.append(row(line, "no", "error", String(error.message ?? error), []));
      }
    }
  }
}

try {
  await mangel.init();
  for (const field of mangel.fields()) option($("field"), field, field.replaceAll("_", " "), "size");
  for (const profile of mangel.profiles()) {
    option($("profile"), profile, profile.replaceAll("_", " "), "name_scrubbing");
  }
  for (const language of mangel.languages()) {
    if (language.output) option($("output"), language.code, `${language.name} (${language.code})`, "sv");
    const box = document.createElement("label");
    const input = document.createElement("input");
    input.type = "checkbox";
    input.value = language.code;
    box.append(input, `${language.name}${language.output ? "" : " (read only)"}`);
    $("label").append(box);
  }
  for (const id of ["values", "field", "profile", "output", "category", "label"]) {
    $(id).addEventListener(id === "label" ? "change" : "input", render);
  }
  $("status").textContent = "Using the local build.";
  render();
} catch (error) {
  $("status").textContent =
    "The WebAssembly build did not load. Build it first: cd mangel-wasm/npm && npm run build. " +
    `(${error.message ?? error})`;
}
