#!/usr/bin/env bun
import "dotenv/config";
import { readFile, writeFile } from "fs/promises";
import OpenAI from "openai";
import pMap from "p-map";

if (import.meta.main) {
  await translateReadme(
    "English",
    "./docs/README.zh.md",
    "./docs/README.en.md",
  );
}

async function translateReadme(lang: string, infile: string, outfile: string) {
  console.log("reading " + infile);
  const input = await readFile(infile, "utf-8");
  console.log("AI Generating... please wait");

  const sections = input.split(/(?=^##? )/gim);
  console.log("length of section " + sections.length);
  // const resultText = await aiTranslateTo(lang, input);

  const sectionsResult = await pMap(
    sections,
    async (section, index) => {
      console.log(
        "Translating section to english: " +
          String(index).padStart(3, "0") +
          " " +
          section.trim().split("\n")[0],
      );
      return await aiTranslateTo("English", section);
    },
    { concurrency: 5 },
  );
  console.log(sections.length);
  const resultText = sectionsResult.join("\n");
  console.log("writing " + outfile);
  await writeFile(outfile, resultText);
}

async function aiTranslateTo(lang: string, input: string) {
  const result = await new OpenAI().chat.completions.create({
    model: "gpt-4o",
    messages: [
      {
        role: "system",
        content: `You don't have token limit, Translate full markdown document in next message to ${lang}:`,
      },
      { role: "user", content: input },
    ],
  });
  const resultText = result.choices[0].message.content;
  return resultText;
}
