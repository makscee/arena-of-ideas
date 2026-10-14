// The card sentence of each rules v2 unit (#1047, Maks's note 1 on batch 1):
// the design's sentence as a person would say it, EN and RU, sleeping and
// awoken. The word chain (form-text.ts) stays for the icons, the glossary and
// the trace; the card and the sheet read this. Numbers that come from the
// tune are not written here: a sentence says "its power", not a value.
// A sentence never starts with a bare number or a status name (units-v2.test.ts).

export interface Say {
  en: string;
  ru: string;
}

/** [sleeping EN, awoken EN, sleeping RU, awoken RU] by unit name. */
export const V2_SAY: Record<string, (tree: { hp: number; awokenHp: number }) => [string, string, string, string]> = {
  Hourglass: () => [
    "At each turn end it gains a grain; at 3 grains, the ally behind it acts again.",
    "At each turn end it gains a grain; at 3 grains, every ally behind it acts again.",
    "В конце каждого хода получает песчинку; на 3 песчинках союзник позади действует ещё раз.",
    "В конце каждого хода получает песчинку; на 3 песчинках все союзники позади действуют ещё раз.",
  ],
  Mirror: () => [
    "At battle start, copies the rule of the enemy in front of it.",
    "At battle start, copies the rule and the power of the enemy in front of it.",
    "В начале боя копирует правило врага перед собой.",
    "В начале боя копирует правило и силу врага перед собой.",
  ],
  Candle: () => [
    "Instead of dying, it stays at 1 HP and loses 1 power.",
    "Instead of dying, it stays at 1 HP, loses 1 power and heals its neighbours by its power.",
    "Вместо смерти остаётся с 1 ОЗ и теряет 1 АТК.",
    "Вместо смерти остаётся с 1 ОЗ, теряет 1 АТК и лечит соседей на свою АТК.",
  ],
  Gambler: () => [
    "Each of its strikes is a coin flip: heads deals double, tails deals nothing.",
    "Each of its strikes is a coin flip: heads deals double and strikes again, tails deals nothing.",
    "Каждая его атака — бросок монеты: орёл — двойной урон, решка — никакого.",
    "Каждая его атака — бросок монеты: орёл — двойной урон и ещё одна атака, решка — никакого.",
  ],
  Nautilus: () => [
    "Each time it is hit, it gains Shield equal to the hits it has taken.",
    "Each time it is hit, it gains Shield equal to the hits it and its allies have taken.",
    "Каждый раз, когда его бьют, получает Щит, равный числу полученных ударов.",
    "Каждый раз, когда его бьют, получает Щит, равный числу ударов по нему и его союзникам.",
  ],
  Magnet: () => [
    "Pulls random hits and statuses aimed at its allies onto itself.",
    "Pulls random hits and statuses aimed at its allies onto itself, and gains 1 Strength each time.",
    "Притягивает на себя случайные удары и статусы, направленные в союзников.",
    "Притягивает на себя случайные удары и статусы, направленные в союзников, и каждый раз получает 1 Силу.",
  ],
  Raven: () => [
    "When an ally dies, it remembers that ally's rule; at each turn start, it performs one remembered rule.",
    "When an ally dies, it remembers that ally's rule; at each turn start, it performs every remembered rule.",
    "Когда союзник погибает, запоминает его правило; в начале каждого хода исполняет одно запомненное правило.",
    "Когда союзник погибает, запоминает его правило; в начале каждого хода исполняет все запомненные правила.",
  ],
  Seed: ({ hp, awokenHp }) => [
    `After two turns, it grows into a Tree (0/${hp}) that gives the ally behind it 1 Strength each turn.`,
    `After two turns, it grows into a Tree (0/${awokenHp}) that gives the ally behind it 1 Strength and its neighbours 1 Shield each turn.`,
    `Через два хода вырастает в Дерево (0/${hp}), которое каждый ход даёт союзнику позади 1 Силу.`,
    `Через два хода вырастает в Дерево (0/${awokenHp}), которое каждый ход даёт союзнику позади 1 Силу, а соседям 1 Щит.`,
  ],
  Sniper: () => [
    "At each turn start, shoots the enemy at the back for its power.",
    "At each turn start, shoots the enemies at the back and at the front for its power.",
    "В начале каждого хода стреляет в заднего врага на свою АТК.",
    "В начале каждого хода стреляет в заднего и переднего врага на свою АТК.",
  ],
  Medic: () => [
    "When an ally is hit, heals it by its power.",
    "When an ally is hit, heals it by its power and gives it 1 Shield.",
    "Когда союзника бьют, лечит его на свою АТК.",
    "Когда союзника бьют, лечит его на свою АТК и даёт ему 1 Щит.",
  ],
  Leech: () => [
    "Each of its strikes heals it by half its power.",
    "Each of its strikes heals it by its full power.",
    "Каждая его атака лечит его на половину АТК.",
    "Каждая его атака лечит его на всю АТК.",
  ],
  Metronome: () => [
    "Every second turn, gives the ally behind it 1 Strength.",
    "Every second turn, gives every ally behind it 1 Strength.",
    "Каждый второй ход даёт союзнику позади 1 Силу.",
    "Каждый второй ход даёт всем союзникам позади 1 Силу.",
  ],
  "Lone Wolf": () => [
    "With no allies left, its strikes deal double.",
    "With no allies left, its strikes deal double and it strikes twice.",
    "Когда союзников не осталось, его атаки наносят двойной урон.",
    "Когда союзников не осталось, его атаки наносят двойной урон, и он бьёт дважды.",
  ],
  Drummer: () => [
    "Each ally strike adds a beat; at 3 beats, every ally behind it gains 1 Strength.",
    "Each ally strike adds a beat; at 3 beats, every ally gains 1 Strength.",
    "Каждая атака союзника добавляет такт; на 3 тактах все союзники позади получают 1 Силу.",
    "Каждая атака союзника добавляет такт; на 3 тактах все союзники получают 1 Силу.",
  ],
  Bodyguard: () => [
    "Once a turn, takes a hit meant for an ally.",
    "Once a turn, takes a hit meant for an ally and gives that ally 1 Shield.",
    "Раз за ход принимает удар, предназначенный союзнику.",
    "Раз за ход принимает удар, предназначенный союзнику, и даёт этому союзнику 1 Щит.",
  ],
  Chrysalis: () => [
    "On the third turn, becomes a Butterfly (2/6) that strikes every enemy for its power each turn.",
    "On the third turn, becomes a Monarch (2/6) that strikes every enemy and gives every ally 1 Strength each turn.",
    "На третьем ходу превращается в Бабочку (2/6), которая каждый ход бьёт всех врагов на свою АТК.",
    "На третьем ходу превращается в Монарха (2/6), который каждый ход бьёт всех врагов и даёт всем союзникам 1 Силу.",
  ],
  Trickster: () => [
    "At battle start, trades rules with the enemy in front of it.",
    "At battle start, trades rules with the enemies at the front and at the back.",
    "В начале боя меняется правилами с врагом перед собой.",
    "В начале боя меняется правилами с передним и задним врагом.",
  ],
  Understudy: () => [
    "The first time an ally dies, it takes over that ally's rule for good.",
    "The first time an ally dies, it takes over that ally's rule and power for good.",
    "Когда впервые погибает союзник, навсегда забирает его правило.",
    "Когда впервые погибает союзник, навсегда забирает его правило и силу.",
  ],
  Gravedigger: () => [
    "At each turn start, hits the enemy in front for its power plus 1 per fallen ally.",
    "At each turn start, hits every enemy for its power plus 1 per fallen ally.",
    "В начале каждого хода бьёт переднего врага на свою АТК плюс 1 за каждого павшего союзника.",
    "В начале каждого хода бьёт всех врагов на свою АТК плюс 1 за каждого павшего союзника.",
  ],
  Conductor: () => [
    "Every second turn, its neighbours act again.",
    "Every second turn, its neighbours act again and gain 1 Strength.",
    "Каждый второй ход его соседи действуют ещё раз.",
    "Каждый второй ход его соседи действуют ещё раз и получают 1 Силу.",
  ],
};
