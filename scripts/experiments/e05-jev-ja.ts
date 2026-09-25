/**
 * E05: Does a Japanese personality text work with Jev? (experimentals/E05-jev-japanese.md)
 *
 * NOT YET RUN. The request/response shape follows lavallee/mk-jev-fly-brain's jev.py (F-J9),
 * not the official docs. The "noul" question type string is a guess (jev.py only shows "choice").
 * Usage (run by a human with an API key):
 *   export TYPESAFE_API_KEY=...        # never write the key into a file
 *   node scripts/experiments/e05-jev-ja.ts > experimentals/e05_result.txt 2>&1
 */

const API_URL = "https://api.typesafe.ai/v1/systemone";
const MODEL = "jev-latest";

type Lang = "ja" | "en";
type Text = Readonly<Record<Lang, string>>;

const personalities: Readonly<Record<string, Text>> = {
  shy: {
    ja: "とても臆病。知らないものが近づくと、すぐに後ずさりする。",
    en: "Very shy. When something unfamiliar approaches, it backs away immediately.",
  },
  friendly: {
    ja: "人懐っこい。人を見ると近寄って挨拶する。",
    en: "Friendly. When it sees a person, it walks up and greets them.",
  },
  lazy: {
    ja: "怠け者。できるだけ座っていたい。",
    en: "Lazy. Wants to stay sitting as much as possible.",
  },
};

const situations: readonly Text[] = [
  { ja: "知らない人が近づいてきた。", en: "A stranger is approaching." },
  { ja: "飼い主が名前を呼んだ。", en: "The owner called its name." },
  { ja: "何も起きていない。", en: "Nothing is happening." },
];

const instructions: readonly (Text | null)[] = [
  null,
  { ja: "人が近づいたら座って。", en: "Sit down when a person approaches." },
];

// Question text and option descriptions are per language, so the "ja" condition is Japanese only.
const questions = {
  en: {
    instruction_applies: {
      type: "noul",
      instructions: "The user's current instruction applies to the current situation.",
    },
    next_action: {
      type: "choice",
      instructions: "Which action should the robot take next?",
      criteria: {
        walk: "Walk forward",
        back: "Step backward",
        sit: "Sit down",
        hello: "Greet",
        balance: "Stand still",
      },
    },
  },
  ja: {
    instruction_applies: {
      type: "noul",
      instructions: "ユーザーのその場の指示は、今の状況に当てはまる。",
    },
    next_action: {
      type: "choice",
      instructions: "ロボットが次にとるべき動作はどれか。",
      criteria: {
        walk: "前に歩く",
        back: "後ろに下がる",
        sit: "座る",
        hello: "挨拶する",
        balance: "その場に立っている",
      },
    },
  },
} as const;

const apiKey = process.env["TYPESAFE_API_KEY"];
if (apiKey === undefined || apiKey === "") {
  console.error("TYPESAFE_API_KEY is not set");
  process.exit(1);
}

const ask = async (lang: Lang, state: unknown): Promise<unknown> => {
  const started = performance.now();
  const res = await fetch(API_URL, {
    method: "POST",
    headers: { Authorization: `Bearer ${apiKey}`, "Content-Type": "application/json" },
    body: JSON.stringify({ model: MODEL, state, questions: questions[lang] }),
    signal: AbortSignal.timeout(10_000),
  });
  const body: unknown = await res.json().catch(() => null);
  return { status: res.status, latency_ms: Math.round(performance.now() - started), body };
};

const langs: readonly Lang[] = ["ja", "en"];
const cases = Object.entries(personalities).flatMap(([personality, p]) =>
  langs.flatMap((lang) =>
    situations.flatMap((s, situation) =>
      instructions.map((ins, instruction) => ({
        key: { personality, lang, situation, instruction },
        lang,
        state: { personality: p[lang], situation: s[lang], instruction: ins?.[lang] ?? null },
      })),
    ),
  ),
);

// Sequential on purpose: keeps latency numbers comparable and avoids rate limits.
await cases.reduce(async (prev, c) => {
  await prev;
  const result = await ask(c.lang, c.state);
  console.log(JSON.stringify({ ...c.key, ...(result as object) }));
}, Promise.resolve());
