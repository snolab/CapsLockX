import OpenAI from "openai";
import 'dotenv/config';
new OpenAI().chat.completions.create({
  stream: true,
  model: "gpt-4-1106-preview",
  messages: [
    { role: "user", content: "以上烴為国工和採" },
    { role: "user", content: "Translate into English:" },
  ],
});
