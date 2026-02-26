import { parseINI, stringifyINI } from "confbox";
// import { HttpProxyAgent } from 'http-proxy-agent';
import { hotMemo } from "hot-memo";
import { writeFile } from "node:fs/promises";
import { OpenAI } from "openai";
import DIE from "phpdie";
import { clone } from "rambda";
import { sflow } from "sflow";
import { createTimeLogger } from "./createTimeLogger";
const LANGUAGE_NAMES = {
  "zh-Hans": "简体中文",
  "zh-Hant": "繁體中文",
  en: "English",
  fr: "Français",
  de: "Deutsch",
  "es-ES": "Español",
  "ja-JP": "日本語",
  "ko-KR": "한국어",
  "ru-RU": "Русский",
  "pt-BR": "Português (Brasil)",
  "it-IT": "Italiano",
  "nl-NL": "Nederlands",
  "pl-PL": "Polski",
  "tr-TR": "Türkçe",
  "ar-SA": "العربية",
  "hi-IN": "हिन्दी",
  "vi-VN": "Tiếng Việt",
  "id-ID": "Bahasa Indonesia",
  "th-TH": "ภาษาไทย",
  "sv-SE": "Svenska",
  "fi-FI": "Suomi",
  "da-DK": "Dansk",
  "no-NO": "Norsk",
  "cs-CZ": "Čeština",
  "hu-HU": "Magyar",
  "ro-RO": "Română",
  "bg-BG": "Български",
  "uk-UA": "Українська",
  "el-GR": "Ελληνικά",
  "he-IL": "עברית",
  "ms-MY": "Bahasa Melayu",
  "tl-PH": "Filipino",
};

if (import.meta.main) {
  const tlog = createTimeLogger();
  tlog("Start ");
  require("global-agent/bootstrap");

  // const HTTP_PROXY = process.env.http_proxy;
  // console.log('http_proxy', HTTP_PROXY)
  // const httpAgent = !HTTP_PROXY ? undefined : new HttpProxyAgent(HTTP_PROXY);

  const ahks = await hotMemo(() => sflow(new Bun.Glob("./**/*.ahk").scan()).toArray(), []);
  console.log(`Scanning ${ahks.length} AHK files`);
  // console.log(process.env)
  // throw 'check'

  // match all t("...") in ahk code
  const translationKeys = await hotMemo(
    (ahks) =>
      sflow(ahks)
        .map(async (file) => ({ file }))
        .mapAddField("content", ({ file }) => hotMemo((f) => Bun.file(f).text(), [file]))
        .mapAddField("translationKeys", ({ content }) =>
          [...content.matchAll(/\bt\(\s*(?:"([^"]+)"|'([^']+)')\s*\)/g)].map(
            (match) => match.slice(1).filter(Boolean)[0],
          ),
        )
        .filter(({ translationKeys }) => translationKeys.length > 0)
        .flatMap((e) => e.translationKeys)
        .uniq()
        .toArray(),
    [ahks],
  );

  console.log(`Found ${translationKeys.length} translation keys`);

  const locales = await hotMemo(
    (_) => sflow(new Bun.Glob("./Core/locales/lang-*.ini").scan()).toArray(),
    [1],
  );
  console.log(
    `Scanning ${locales.length} locale files: ${locales
      .map((e) => e.match("lang-(.*).ini")[1])
      .join(", ")}`,
  );

  type LocaleFile = Record<string, Record<string, string>>;

  await sflow(locales)
    .map(async (file) => ({ file }))
    .mapAddField(
      "locale",
      ({ file }) => file.match(/lang-(.*)\.ini$/)?.[1] || DIE("Unknown LOCALE"),
    )
    .mapAddField("content", ({ file }) =>
      hotMemo(
        (f) =>
          Bun.file(f)
            .text()
            .catch(() => ""),
        [file, Bun.file(file).lastModified],
      ),
    )
    .mapAddField("res", ({ content, locale }) =>
      hotMemo(
        (c) => ({
          ["lang-" + locale]: {},
          ...(!c ? {} : parseINI<LocaleFile>(c)),
        }),
        [content],
      ),
    )
    .mapAddField("missingKeys", ({ res, locale }) =>
      translationKeys.filter((key) => !res[`lang-${locale}`]?.[key]),
    )
    .mapAddField("unusedKeys", ({ res, locale }) =>
      Object.keys(res[`lang-${locale}`] || {}).filter((key) => !translationKeys.includes(key)),
    )
    .log(
      ({ file, locale, missingKeys, unusedKeys }) =>
        `LOCALE ${locale}: missing ${missingKeys.length}, unused ${unusedKeys.length}`,
    )
    .filter(({ missingKeys, unusedKeys }) => missingKeys.length > 0 || unusedKeys.length > 0)
    .pMap(
      async ({ missingKeys, unusedKeys, locale, res }) => {
        const newRes = clone(res);
        const translations = newRes[`lang-${locale}`];
        // for unsued keys, remove them from res[`lang-${locale}`], and save the file
        unusedKeys.forEach((key) => {
          console.log(`${locale}| - ${translations[key]?.slice(0, 5)}...`);
          delete translations[key];
          return key;
        });
        // for existed keys, trim '"'
        Object.keys(translations).forEach((key) => {
          if (translations[key].startsWith('"') && translations[key].endsWith('"')) {
            translations[key] = translations[key].slice(1, -1);
          }
        });
        await saveLocale(locale, newRes);

        // translate missing keys to target locale using chatgpt-4o
        await sflow(missingKeys)
          .map(async (key) => {
            const transcript = await new OpenAI().chat.completions
              // { httpAgent } // Use HTTP_PROXY if set, otherwise use default
              .create({
                model: "gpt-4o",
                messages: [
                  {
                    role: "system",
                    content: `You are a translation assistant. Translate the following text to target locale/language without any explainations. The input is in JSON, and output is plain text`,
                  },
                  // expamle
                  {
                    role: "user",
                    content: `Translate the following text to ${"zh-CN"} language:\n${JSON.stringify(
                      "Hello, world!",
                    )}`,
                  },
                  { role: "assistant", content: `你好，世界！` },
                  {
                    role: "user",
                    content: `Translate the following text to ${
                      LANGUAGE_NAMES[locale] || locale
                    } language:\n"${JSON.stringify(key)}"`,
                  },
                ],
                temperature: 0.2,
                max_tokens: 100,
              })
              .then((res) => {
                const text = res.choices[0].message.content.trim();
                return text;
              });
            console.log(`${locale}| + ${transcript.slice(0, 5)}...`);
            // trim '"'
            translations[key] = transcript.replace(/^"(.*)"$/g, "$1");
            await saveLocale(locale, newRes);
          })
          .run();

        await saveLocale(locale, newRes).then(() => console.log(`Saved ${locale} locale file`));
        return [];
      },
      { concurrency: 3 },
    )
    .uniq()
    .run();

  tlog("Done");
}
async function saveLocale(locale: string, newRes: { [x: string]: {} }) {
  const iniPath = `./Core/locales/lang-${locale}.ini`;
  const content = await Bun.file(iniPath)
    .text()
    .catch(() => "");
  const newContent = stringifyINI(newRes);
  if (!(newContent === content)) {
    await writeFileUtf16le(iniPath, newContent);
  }
}

async function saveIni(filePath: string, content: { [topic: string]: { [key: string]: string } }) {
  const text = stringifyINI(content);
  await writeFileUtf16le(filePath, text);
}

async function writeFileUtf16le(filePath: string, text: string) {
  const utf16lebom = "\uFEFF"; // UTF-16 le BOM
  await writeFile(filePath, utf16lebom + text, { encoding: "utf16le" });
}
