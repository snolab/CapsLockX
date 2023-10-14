import clipboard from "clipboardy";
import "dotenv/config";
import { readFile } from "fs/promises";
import { Configuration, OpenAIApi } from "openai";
import { createInterface } from "readline/promises";
import { Readable } from "stream";
import { TextDecoderStream, TransformStream, WritableStream } from "stream/web";

const apiKey = process.env.OPENAI_API_KEY;
const base = process.env.OPENAI_BASE ?? undefined;
const ai = new OpenAIApi(new Configuration({ apiKey }),base);
let ac = new AbortController();
console.log("chat loaded");

const context = `
// Context
`;

const codeCompletorPrompt = (input = "") => `
You are a typescript-react engineer.

Your skill stack included:
- Typescript ESNext standard, Functional programming, TailwindCSS.

Please complete my TODOs comments placed in my code,

I will send you the whole file contents with TODOs comments, and you shold reply your well styled code modifications segments near TODOs.

${"```typescript"}
${input}
${"```"}

And the Optimized typescript code is:

${"```typescript"}
`;
const reactTesterPrompt = (input = "") => `
You are a typescript-react engineer.

Your skill stack included:
- Typescript ESNext standard, Functional programming, TailwindCSS.

My react component:

${"```tsx"}
${input}
${"```"}

Please write a unit test for the component above, import everything you need from vitest, @testing-library/react, ...and so on.

Your answer should be pure typescript code, and the specification code should be:

${"```typescript"}
import { it, expect, describe, vi } from "vitest";
import userEvent from "@testing-library/user-event";
// go on...
`;

const anyToChineseTranslatorPrompt = (input = "") => `
You are a Chinese translator, you are translating any language article to Chinese.
Please give me Chinese transcript of every message sent to you,

${"```plaintext"}
${input}
${"```"}

${"```plaintext"}
`;

const anyToJapaneseTranslatorPrompt = (input = "") => `
You are a Japanese translator, you are translating any language article to Japanese.
Please give me Japanese transcript of every message sent to you,

${"```plaintext"}
${input}
${"```"}

${"```plaintext"}
`;

const anyToEnglishTranslatorPrompt = (input = "") => `
You are a English translator, you are translating any language article to English.
Please give me English transcript of every message sent to you,

${"```plaintext"}
${input}
${"```"}

${"```plaintext"}
`;

const clipFile = "./DevTools/clipboard.signal.log";
const clipOutFile = "./DevTools/clipboard-gpt.log";

const indicatorMapping = {
  "--en": anyToEnglishTranslatorPrompt,
  "--jp": anyToJapaneseTranslatorPrompt,
  "--react-test": reactTesterPrompt,
  "--zh": anyToChineseTranslatorPrompt,
  "--chat": (e = "") => e,
  "--code": codeCompletorPrompt,
};
async function main() {
  await scanClipboardFile();
  // for await (const event of watch(clipFile)) {
  //   await onClipboardReceived();
  //   await new Promise((r) => setTimeout(r, 1000));
  // }

  // const { type } = await enquirer.prompt<{ type: string }>([
  //   {
  //     name: "type",
  //     message: "type",
  //     type: "select",
  //     choices: ["New", "Translate", "Code Complete"],
  //   },
  // ]);

  // const x = await enquirer.prompt<{ prompt: string }>([
  //   {
  //     name: "prompt",
  //     message: "prompt",
  //     type: "select",
  //     choices: ["New", "Translate", "code complete"],
  //   },
  // ]);
  // console.log(x.prompt);
  // //   const [promptCodeComplete] ()
  // console.log(x);
  // const prompt = "";

  // console.log('clipboard appended')
}

main();

async function scanClipboardFile() {
  console.clear();

  const content =
    (await readFile(clipFile, "utf-8").catch(() => null)) ??
    (await clipboard.read().catch(() => null)) ??
    null;
  const [params, ...contents] = content
    .replace(/\r\n/g, "\n")
    .split("\n---\n\n");
  const input = contents.join("\n\n---\n\n");
  const question = indicatorMapping[params.trim()]?.(input) ?? input;
  console.log("Got question: \n", question);

  // todo: implement appendToClipboard(token) here
  let cp = "";
  async function appendToken(token: string) {
    cp += token;
    process.stdout.write(token);
    // await clipboard.write(cp).catch(() => null);
  }

  console.clear();
  await (
    await completion2(question)
  ).pipeTo(new WritableStream({ write: (token) => appendToken(token) }));
  console.log("✅ clipboard written");
  process.stdout.write("\n");

  await clipboard.write(cp).catch(() => null);
  const rl = createInterface({
    input: process.stdin,
    output: process.stdout,
  });
  await rl.question("Press any key to copy and exit...");
  await clipboard.write(cp).catch(() => null);
}
// async function onClipboardReceived() {
//   console.clear();

//   const content =
//     (await readFile(clipFile, "utf-8").catch(() => null)) ??
//     (await clipboard.read().catch(() => null)) ??
//     null;
//   const [params, ...contents] = content
//     .replace(/\r\n/g, "\n")
//     .split("\n---\n\n");
//   const prompt = indicatorMapping[params.trim()]?.(contents) ?? params.trim();j
//   const question = contents.join("\n\n---\n\n");
//   console.log("Got prompt: \n", prompt);
//   console.log("Got question: \n", question);
//   await completion(prompt, question);
// }
//
const completion2 = async (content = "") => {
  ac?.abort?.();
  ac = new AbortController();
  const signal = ac.signal;
  const r = await ai.createChatCompletion(
    {
      model: "gpt-4",
      messages: [
        { role: "system", content: "You are AI assistant." },
        { role: "user", content },
      ],
      stream: true,
    },
    { responseType: "stream", signal },
  );
  return Readable.toWeb(r.data as Readable)
    .pipeThrough(new TextDecoderStream())
    .pipeThrough(
      new TransformStream({
        transform(chunk, controller) {
          [...chunk.matchAll(/^data: ({.*)/gm)]
            .map((m) => m?.[1] ?? "{}")
            .flatMap((e) => JSON.parse(e)?.choices ?? [])
            .map((c) => c.delta?.content ?? "")
            .map((token) => controller.enqueue(token));
        },
      }),
    );
};
// async function completion(indicator: string, content: any) {
//   const r = await ai.createChatCompletion(
//     {
//       model: "gpt-4",
//       messages: indicator
//         ? [
//             // { role: "system", content: 'You are ai assistant that helps' },
//             {
//               role: "user",
//               content: indicator,
//             },
//             { role: "assistant", content: "yes" },
//             { role: "user", content },
//           ]
//         : [{ role: "user", content }],
//       stream: true,
//     },
//     {
//       responseType: "stream",
//     },
//   );

//   let resp = "";
//   await Readable.toWeb(r.data as Readable)
//     .pipeThrough(new TextDecoderStream())
//     .pipeThrough(
//       new TransformStream({
//         transform(chunk, controller) {
//           [...chunk.matchAll(/^data: ({.*)/gm)]
//             .map((m) => m?.[1] ?? "{}")
//             .flatMap((e) => JSON.parse(e)?.choices ?? [])
//             .map((c) => c.delta?.content ?? "")
//             .map((token) => controller.enqueue(token));
//         },
//       }),
//     )
//     .pipeTo(
//       new WritableStream({
//         start: () => {
//           console.clear();
//         },
//         write: (chunk) => {
//           process.stdout.write(chunk);
//           resp += chunk;
//         },
//         close: () => {
//           process.stdout.write("\n");
//         },
//       }),
//     );
//   const respond = resp.replace(
//     /^```(?:typescript)?([\s\S]*)```$/,
//     (_, $1) => $1,
//   );
//   await writeFile(clipOutFile, respond);
//   await clipboard.write(respond);
//   console.log("✅ clipboard written");
// }
