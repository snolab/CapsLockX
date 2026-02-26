// WIP
`
Main  Reference: /README.md
Translated Reference: docs/README.*.md


Increment Translate plan:

main_full = Read from main reference file
translated_parts = and read from translated reference file.

Split them into sections, and ask for chatgpt if the section is already the updated translated.
If translated, then append to the final file.
If not translated or outdated, then ask for translation and append to the final file.

`;
import fs from "fs";
import { readFile } from "fs/promises";
import { OpenAI } from "openai";
// Define the paths for the main and translated reference files
const mainFilePath = "README.md";
const translatedFilePath = "docs/README.es.md"; // Assuming Spanish as an example

// Read contents of the main reference file and translated reference file
const mainFull = await readFile(mainFilePath, "utf-8");
const translatedParts = await readFile(translatedFilePath, "utf-8");

// Here, you need to replace "splitIntoSections" with your actual function to split content into sections.
const mainSections = splitReadme(mainFull);
const translatedSections = splitReadme(translatedParts);

// Helper function to check if a section is translated and updated
const translateUpdatedQ = async (translationLanguage, referenceSection, translatedSection) =>
  await new OpenAI().chat.completions
    .create({
      messages: [
        {
          role: "user",
          message: JSON.stringify({
            translationLanguage,
            referenceSection,
            translatedSection,
          }),
        },
        {
          role: "user",
          message:
            'You received a JSON, Is the translation content already updated with reference section? Reply "Yes" or "No", Do not explain other things, do not add any punctuations.',
        },
      ],
      model: "gpt-4o",
    })
    .then((e) => e.choices[0].message.content);

// Helper function to translate a section
const translateSection = async (lang, section) =>
  await new OpenAI().chat.completions
    .create({
      messages: [
        {
          role: "user",
          message: JSON.stringify({ mainSection }),
        },
        { role: "user", message: "Is the translation already updated?" },
      ],
      model: "gpt-4o",
    })
    .then((e) => e.choices[0].message.content);

// Draft the final translated file
const draftFinalFile = async () => {
  let finalContent = "";

  for (let i = 0; i < mainSections.length; i++) {
    let mainSection = mainSections[i];
    let translatedSection = translatedSections[i];

    if (isSectionTranslatedAndUpdated(mainSection, translatedSection)) {
      finalContent += translatedSection;
    } else {
      const newSection = await translateSection(mainSection);
      finalContent += newSection;
      // Optionally update the translatedSections array if necessary
      translatedSections[i] = newSection;
    }
  }

  // Save the final content to a new file or overwrite the existing translated file
  fs.writeFileSync(translatedFilePath, finalContent, "utf-8");
};

// await draftFinalFile();

// js function split readme file into sections by # and ##
function splitReadme(content) {
  const sections = content.split(/\n(?=##(?!#))/).reduce((acc, section) => {
    const trimmed = section.trim();
    if (trimmed.startsWith("#")) {
      acc.push(trimmed);
    }
    return acc;
  }, []);
  return sections;
}
console.log(splitReadme(mainFull).map((e) => [e.length, e.split("\n")[0]]));
