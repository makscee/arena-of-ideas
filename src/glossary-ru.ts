// The glossary in Russian (M4-3): every term's label and rule, in the words
// describe-ru.ts writes card text with ("Удар", "Начало боя", "АТК"). Icons
// and tones stay glossary.ts's; only the words change. Display-only.

import { FATIGUE_START, fatigueAmount } from "./battle.js";
import { RU_STATUSES, RU_STAT } from "./describe-ru.js";
import type { EventPattern, UnitFilter } from "./types.js";

/** A term's Russian words. */
export interface RuTerm {
  label: string;
  tip: string;
  more?: string;
}

/** The shipped statuses' rules (labels come from describe-ru.ts's table). */
export const RU_STATUS_TIPS: Record<string, string> = {
  Shield: "Блокирует любой урон: каждая заблокированная единица тратит 1 заряд.",
  Vitality: "+1 ОЗ за заряд, до конца боя.",
  Strength: "+1 АТК за заряд, до конца боя.",
  Curse: "−1 АТК за заряд, до конца боя (АТК не ниже 0).",
  Poison: "В конце каждого хода наносит урон, равный числу зарядов, затем теряет 1 заряд.",
  Freeze: "Пропускает следующую атаку; каждая пропущенная атака тратит 1 заряд.",
  Blessing: "Когда носитель должен умереть, он остаётся жить с ОЗ, равными числу зарядов, и теряет их все.",
};

export const ruStatusTerm = (name: string): RuTerm | undefined => {
  const s = RU_STATUSES[name];
  const tip = RU_STATUS_TIPS[name];
  return s && tip ? { label: s.name, tip } : undefined;
};

/** Russian words for every fixed term (glossary.ts's GLOSSARY keys). The
 * battle terms whose tips carry numbers are filled in by glossary.ts. */
export const RU_TERMS: Record<string, RuTerm> = {
  "stat:pwr": { label: RU_STAT.pwr, tip: "Атака: урон, который юнит наносит каждой атакой." },
  "stat:hp": { label: RU_STAT.hp, tip: "Очки здоровья. Когда они доходят до 0, юнит умирает." },

  "trigger:BattleStart": { label: "Начало боя", tip: "Один раз, в начале боя, до первой атаки." },
  "trigger:TurnStart": { label: "Начало хода", tip: "В начале каждого хода." },
  "trigger:TurnEnd": { label: "Конец хода", tip: "В конце каждого хода, после атак передних юнитов." },
  "trigger:Strike": { label: "Атака", tip: "Когда юнит делает обычную атаку. Каждый ход два передних юнита атакуют друг друга." },
  "trigger:Hurt": { label: "Удар", tip: "Когда в юнита приходит любой урон, от чего угодно. Считается, даже если весь урон заблокирован." },
  "trigger:Heal": { label: "Лечение", tip: "Когда юнит восстанавливает ОЗ. Юнита с полными ОЗ вылечить нельзя, тогда это не срабатывает." },
  "trigger:Death": { label: "Смерть", tip: "Когда юнит умирает. Его способность на смерть всё равно срабатывает, когда он покидает строй." },
  "trigger:Summon": { label: "Призыв", tip: "Когда юнит встаёт в строй посреди боя, новый или возвращённый." },
  "trigger:StatusApplied": { label: "Получает статус", tip: "Когда на юнита накладывают этот статус." },
  "trigger:StatusRemoved": { label: "Теряет статус", tip: "Когда этот статус израсходован или снят." },
  "trigger:StatChanged": { label: "Набирает АТК", tip: "Когда АТК юнита растёт, от чего угодно." },
  "term:would": { label: "перед", tip: "«Перед …» срабатывает прямо перед событием и может изменить или остановить его." },

  "condition:holderHpAtMost": { label: "Мало ОЗ", tip: "Срабатывает, только пока его ОЗ не выше числа." },

  "target:holder": { label: "Себе", tip: "Юнит с этой способностью." },
  "target:eventUnit": { label: "Ему", tip: "Юнит, о котором было событие." },
  "target:attacker": { label: "Атакующему", tip: "Тот, кто нанёс удар, атакой или способностью. У урона от статуса со временем атакующего нет, а погибший атакующий недосягаем: тогда ничего не происходит." },
  "target:frontEnemy": { label: "Переднему врагу", tip: "Первый враг в строю, тот, кто сражается сейчас." },
  "target:randomEnemy": { label: "Случайному врагу", tip: "Один живой враг, выбранный случайно." },
  "target:allEnemies": { label: "Всем врагам", tip: "Каждый живой враг." },
  "target:allAllies": { label: "Всем союзникам", tip: "Каждый живой союзник, включая себя: юнит с этой способностью тоже считается." },
  "target:lastDeadAlly": { label: "Павшему союзнику", tip: "Союзник, который умер последним и всё ещё мёртв." },

  "effect:damage": { label: "Урон", tip: "Отнимает столько ОЗ." },
  "effect:heal": { label: "Лечение", tip: "Возвращает потерянные ОЗ, не выше максимума юнита." },
  "effect:applyStatus": { label: "Наложить статус", tip: "Накладывает заряды статуса на цель." },
  "effect:consumeStacks": { label: "Потратить", tip: "Снимает заряды статуса с юнита, у которого он есть." },
  "effect:summon": { label: "Призвать", tip: "Ставит нового юнита в начало строя, если в строю есть место (не больше 5). Числа: его АТК / ОЗ." },
  "effect:resurrect": { label: "Воскресить", tip: "Возвращает павшего союзника в конец строя с этим числом ОЗ, если есть место." },
  "effect:silence": { label: "Заглушить", tip: "Снимает все его статусы и выключает его способности до конца боя." },
  "effect:cancel": { label: "Отменить", tip: "Останавливает то, что вот-вот должно было случиться." },
  "effect:absorbHurt": { label: "Поглотить", tip: "Блокирует урон до числа зарядов, тратя то, что заблокировал." },
  "effect:preventDeathHeal": { label: "Обмануть смерть", tip: "Останавливает смерть и вместо неё задаёт юниту ОЗ." },

  "state:sleeping": { label: "Спящий", tip: "Первая форма юнита. 3 копии пробуждают его." },
  "state:awoken": { label: "Пробуждённый", tip: "Сильная форма, открывается 3-й копией." },
  "state:fused": { label: "Слитый", tip: "Два пробуждённых юнита в одном: Когда первого, Кому второго, Что делают — оба." },

  "battle:fatigue": { label: "Усталость", tip: "" },
  "battle:suddenDeath": { label: "Внезапная смерть", tip: "" },
  "battle:chainCapped": { label: "Цепь прервана", tip: "" },
  "battle:noRoom": { label: "Нет места", tip: "Когда строй полон, юнит, который должен встать в него посреди боя, не появляется: ничего не происходит." },
  "battle:timeUp": { label: "Время вышло", tip: "" },

  "term:stacks": { label: "Заряды", tip: "Число рядом со статусом. Больше зарядов, сильнее эффект; некоторые статусы тратят заряды." },
};

export function chainCappedTipRu(cap: number): string {
  return `Цепь реакций прошла ${cap} шагов и была прервана, чтобы бой не зациклился.`;
}

export function fatigueTipRu(suddenDeathAt?: number): string {
  const n = (t: number) => fatigueAmount(t, suddenDeathAt);
  const linear = `С хода ${FATIGUE_START} каждый юнит получает урон в конце каждого хода: ${n(FATIGUE_START)}, затем ${n(FATIGUE_START + 1)}, ${n(FATIGUE_START + 2)} …`;
  if (suddenDeathAt === undefined) return `${linear} поэтому бой всегда заканчивается.`;
  return `${linear} С хода ${suddenDeathAt} он удваивается каждый ход, поэтому бой всегда заканчивается.`;
}

export function suddenDeathTipRu(at: number): string {
  const n = (t: number) => fatigueAmount(t, at);
  return `С хода ${at} урон в конце хода удваивается каждый ход (${n(at)}, ${n(at + 1)}, ${n(at + 2)} …), его ничто не блокирует и ничто от него не спасает, и ни один юнит не может встать или вернуться в строй.`;
}

export function timeUpTipRu(cap: number): string {
  return `Если после хода ${cap} обе стороны ещё стоят, бой заканчивается ничьей, чтобы бой не длился вечно.`;
}

/** A unit trigger said of someone else (glossary.ts's SCOPED_EVENT). */
export const RU_SCOPED_EVENT: Partial<Record<EventPattern["on"], string>> = {
  Strike: "делает обычную атаку.",
  Hurt: "получает любой урон, от чего угодно. Считается, даже если весь урон заблокирован.",
  Heal: "восстанавливает ОЗ. Юнита с полными ОЗ вылечить нельзя, тогда это не срабатывает.",
  Death: "умирает.",
  Summon: "встаёт в строй посреди боя, новый или возвращённый.",
  StatusApplied: "получает этот статус.",
  StatusRemoved: "теряет этот статус: израсходован или снят.",
  StatChanged: "набирает АТК, от чего угодно.",
};

export const RU_SCOPE_WHO: Record<Exclude<UnitFilter, "holder">, string> = {
  ally: "Когда любой союзник, включая себя,",
  otherAlly: "Когда другой союзник (не сам юнит)",
  enemy: "Когда враг",
  any: "Когда любой юнит, с любой стороны,",
};

/** Whose event, as a label opens or ends it: "Смерть союзника", "Удар по
 * врагу", "Союзник получает Щит". */
const WHO: Record<Exclude<UnitFilter, "holder">, { nom: string; gen: string; dat: string }> = {
  ally: { nom: "Союзник", gen: "союзника", dat: "союзнику" },
  otherAlly: { nom: "Союзник", gen: "союзника", dat: "союзнику" },
  enemy: { nom: "Враг", gen: "врага", dat: "врагу" },
  any: { nom: "Любой", gen: "любого", dat: "любому" },
};

/** A trigger's Russian label said of a scope, the card's way: event nouns
 * take the unit after them ("Смерть союзника", "Удар по врагу"), verbs before
 * ("Союзник получает Щит"). */
export function scopeLabelRu(on: EventPattern["on"] | undefined, label: string, scope?: UnitFilter): string {
  if (!scope || scope === "holder") return label;
  const w = WHO[scope];
  if (on === "Hurt") return `${label} по ${w.dat}`;
  if (on === "Strike" || on === "Heal" || on === "Death" || on === "Summon") return `${label} ${w.gen}`;
  return `${w.nom} ${label.charAt(0).toLowerCase()}${label.slice(1)}`;
}
