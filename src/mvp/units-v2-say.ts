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
    "At each turn end it gains 1 grain; once it has 3, it spends them and the ally behind it acts again (turn ends 3, 6, 9…).",
    "At each turn end it gains 1 grain; once it has 3, it spends them and every ally behind it acts again (turn ends 3, 6, 9…).",
    "В конце каждого хода получает 1 песчинку; набрав 3, тратит их, и союзник позади действует ещё раз (в конце 3-го, 6-го, 9-го… хода).",
    "В конце каждого хода получает 1 песчинку; набрав 3, тратит их, и все союзники позади действуют ещё раз (в конце 3-го, 6-го, 9-го… хода).",
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
    "Each of its strikes has a 50% chance to deal double; otherwise it deals nothing.",
    "Each of its strikes has a 50% chance to deal double and strike again; otherwise it deals nothing.",
    "Каждая его атака с вероятностью 50% наносит двойной урон, иначе — никакого.",
    "Каждая его атака с вероятностью 50% наносит двойной урон и бьёт ещё раз, иначе — никакого.",
  ],
  Nautilus: () => [
    "Each time it is hit, it gains 1 Shield.",
    "Each time it is hit, it gains 2 Shield.",
    "Каждый раз, когда его бьют, получает 1 Щит.",
    "Каждый раз, когда его бьют, получает 2 Щита.",
  ],
  Magnet: () => [
    "Pulls random hits and statuses aimed at its allies onto itself.",
    "Pulls random hits and statuses aimed at its allies onto itself, and gains 1 Strength each time.",
    "Притягивает на себя случайные удары и статусы, направленные в союзников.",
    "Притягивает на себя случайные удары и статусы, направленные в союзников, и каждый раз получает 1 Силу.",
  ],
  Raven: () => [
    "When an ally dies, it remembers that ally's rule; at each turn start, it performs its oldest remembered rule once and forgets it.",
    "When an ally dies, it remembers that ally's rule; at each turn start, it performs every remembered rule.",
    "Когда союзник погибает, запоминает его правило; в начале каждого хода исполняет самое старое запомненное правило и забывает его.",
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
    "At the start of turns 2, 4, 6…, it gives the ally behind it 1 Strength.",
    "At the start of turns 2, 4, 6…, it gives every ally behind it 1 Strength.",
    "В начале 2-го, 4-го, 6-го… хода даёт союзнику позади 1 Силу.",
    "В начале 2-го, 4-го, 6-го… хода даёт всем союзникам позади 1 Силу.",
  ],
  "Lone Wolf": () => [
    "With no allies left, its strikes deal double.",
    "With no allies left, its strikes deal double and it strikes twice.",
    "Когда союзников не осталось, его атаки наносят двойной урон.",
    "Когда союзников не осталось, его атаки наносят двойной урон, и он бьёт дважды.",
  ],
  Drummer: () => [
    "Each time an ally strikes, it gains 1 beat; at a turn end with 3+ beats, it spends 3 and every ally behind it gains 1 Strength.",
    "Each time an ally strikes, it gains 1 beat; at a turn end with 3+ beats, it spends 3 and every ally gains 1 Strength.",
    "Каждый раз, когда союзник атакует, получает 1 такт; в конце хода, если тактов 3 или больше, тратит 3, и все союзники позади получают 1 Силу.",
    "Каждый раз, когда союзник атакует, получает 1 такт; в конце хода, если тактов 3 или больше, тратит 3, и все союзники получают 1 Силу.",
  ],
  Bodyguard: () => [
    "The first time each turn a hit is aimed at an ally, it takes that hit instead.",
    "The first time each turn a hit is aimed at an ally, it takes that hit instead and gives that ally 1 Shield.",
    "Первый за ход удар по союзнику принимает на себя.",
    "Первый за ход удар по союзнику принимает на себя и даёт этому союзнику 1 Щит.",
  ],
  Chrysalis: () => [
    "At the start of turn 3, it becomes a Butterfly (2/4) that strikes every enemy for its power at each turn start.",
    "At the start of turn 3, it becomes a Monarch (2/4) that, at each turn start, strikes every enemy and gives every ally 1 Strength.",
    "В начале 3-го хода превращается в Бабочку (2/4), которая в начале каждого хода бьёт всех врагов на свою АТК.",
    "В начале 3-го хода превращается в Монарха (2/4), который в начале каждого хода бьёт всех врагов и даёт всем союзникам 1 Силу.",
  ],
  Trickster: () => [
    "At battle start, trades rules with the enemy in front of it.",
    "At battle start, trades rules with the enemies at the front and at the back.",
    "В начале боя меняется правилами с врагом перед собой.",
    "В начале боя меняется правилами с передним и задним врагом.",
  ],
  Understudy: () => [
    "The first time an ally dies in a battle, it takes over that ally's rule for good.",
    "The first time an ally dies in a battle, it takes over that ally's rule and power for good.",
    "Когда в бою впервые погибает союзник, навсегда забирает его правило.",
    "Когда в бою впервые погибает союзник, навсегда забирает его правило и силу.",
  ],
  Gravedigger: () => [
    "At each turn start, hits the enemy in front for 1 per fallen ally.",
    "At each turn start, hits every enemy for 1 per fallen ally.",
    "В начале каждого хода бьёт переднего врага на 1 за каждого павшего союзника.",
    "В начале каждого хода бьёт всех врагов на 1 за каждого павшего союзника.",
  ],
  Conductor: () => [
    "At the start of turns 3, 6, 9…, its neighbours act again.",
    "At the start of turns 3, 6, 9…, its neighbours act again and gain 1 Strength.",
    "В начале 3-го, 6-го, 9-го… хода его соседи действуют ещё раз.",
    "В начале 3-го, 6-го, 9-го… хода его соседи действуют ещё раз и получают 1 Силу.",
  ],
  // batch 2 (#1047)
  Thornback: () => [
    "When it is hit, it pricks its attacker for 1.",
    "When it is hit, it pricks its attacker for its power.",
    "Когда его бьют, колет обидчика на 1.",
    "Когда его бьют, колет обидчика на свою АТК.",
  ],
  "Coin Thief": () => [
    "At each turn start, 50% chance: it robs a random enemy for its power.",
    "At each turn start, 75% chance: it robs a random enemy for its power.",
    "В начале каждого хода с вероятностью 50% грабит случайного врага на свою АТК.",
    "В начале каждого хода с вероятностью 75% грабит случайного врага на свою АТК.",
  ],
  Scavenger: () => [
    "When an ally falls, it gains 1 Strength.",
    "When an ally falls, it gains 1 Strength and heals by its power.",
    "Когда союзник падает, получает 1 Силу.",
    "Когда союзник падает, получает 1 Силу и лечится на свою АТК.",
  ],
  Lantern: () => [
    "At each turn start, if it is at full HP, it gives the ally ahead of it 1 Shield.",
    "At each turn start, if it is at full HP, it gives both its neighbours 1 Shield.",
    "В начале каждого хода, если у него полные ОЗ, даёт союзнику впереди 1 Щит.",
    "В начале каждого хода, если у него полные ОЗ, даёт обоим соседям 1 Щит.",
  ],
  Hornet: () => [
    "Each of its strikes poisons the enemy it hits.",
    "Each of its strikes poisons every enemy.",
    "Каждая его атака отравляет того, кого он бьёт.",
    "Каждая его атака отравляет всех врагов.",
  ],
  "Echo Bell": () => [
    "At battle start, it learns the rule of the ally behind it; at the start of turns 2, 4, 6…, it performs that rule once.",
    "At battle start, it learns the rule of the ally behind it; at each turn start, it performs that rule once.",
    "В начале боя запоминает правило союзника позади; в начале 2-го, 4-го, 6-го… хода исполняет его один раз.",
    "В начале боя запоминает правило союзника позади; в начале каждого хода исполняет его один раз.",
  ],
  Prism: () => [
    "A random hit aimed at it bends onto a random enemy instead.",
    "A random hit aimed at it bends onto a random enemy instead, and it gains 1 Shield.",
    "Случайный удар по нему вместо этого уходит в случайного врага.",
    "Случайный удар по нему вместо этого уходит в случайного врага, и он получает 1 Щит.",
  ],
  "Wolf Pup": () => [
    "The first time an ally falls in a battle, it grows into a Dire Wolf (4/10) that gains 2 Strength at each turn end.",
    "The first time an ally falls in a battle, it grows into a Dire Wolf (5/12) that gains 2 Strength at each turn end.",
    "Когда в бою впервые падает союзник, вырастает в Лютоволка (4/10), который в конце каждого хода получает 2 Силы.",
    "Когда в бою впервые падает союзник, вырастает в Лютоволка (5/12), который в конце каждого хода получает 2 Силы.",
  ],
  "War Horn": () => [
    "At each turn start, with 3 or fewer allies standing, it gives every ally 1 Strength.",
    "At each turn start, with 3 or fewer allies standing, it gives every ally 1 Strength and 1 Shield.",
    "В начале каждого хода, если союзников осталось 3 или меньше, даёт всем союзникам 1 Силу.",
    "В начале каждого хода, если союзников осталось 3 или меньше, даёт всем союзникам 1 Силу и 1 Щит.",
  ],
  Copycat: () => [
    "At battle start, copies the rule of the ally ahead of it.",
    "At battle start, copies the rule and the power of the ally ahead of it.",
    "В начале боя копирует правило союзника впереди.",
    "В начале боя копирует правило и силу союзника впереди.",
  ],
  Vulture: () => [
    "When an enemy falls, it gains 1 Strength.",
    "When an enemy falls, every ally gains 1 Strength.",
    "Когда падает враг, получает 1 Силу.",
    "Когда падает враг, все союзники получают 1 Силу.",
  ],
  Hexer: () => [
    "At the start of turn 2, silences the enemy at the back: it loses its rule and statuses.",
    "At the start of turn 2, silences the enemies at the back and at the front: they lose their rules and statuses.",
    "В начале 2-го хода заглушает заднего врага: тот теряет правило и статусы.",
    "В начале 2-го хода заглушает заднего и переднего врага: они теряют правила и статусы.",
  ],
  "Fortune Teller": () => [
    "At battle start, 50% chance: every ally gains 1 Strength; otherwise every ally gains 1 Shield.",
    "At battle start, 50% chance: every ally gains 2 Strength; otherwise every ally gains 2 Shield.",
    "В начале боя с вероятностью 50% все союзники получают 1 Силу, иначе — 1 Щит.",
    "В начале боя с вероятностью 50% все союзники получают 2 Силы, иначе — 2 Щита.",
  ],
  Aegis: () => [
    "The first hit on it each turn deals nothing.",
    "The first hit on it each turn deals nothing, and the ally behind it gains 1 Shield.",
    "Первый удар по нему за каждый ход не наносит урона.",
    "Первый удар по нему за каждый ход не наносит урона, а союзник позади получает 1 Щит.",
  ],
  Gravecaller: () => [
    "The first time an ally falls in a battle, it calls that ally back with 1 HP.",
    "The first time an ally falls in a battle, it calls that ally back with HP equal to its power.",
    "Когда в бою впервые падает союзник, возвращает его с 1 ОЗ.",
    "Когда в бою впервые падает союзник, возвращает его с ОЗ, равными своей АТК.",
  ],
  Jester: () => [
    "The first time it is hit in a battle, it trades rules with its attacker.",
    "The first time it is hit in a battle, it trades rules with its attacker and gains 1 Shield.",
    "Когда его впервые за бой бьют, меняется правилами с обидчиком.",
    "Когда его впервые за бой бьют, меняется правилами с обидчиком и получает 1 Щит.",
  ],
  Hydra: () => [
    "When it dies, two Heads (2/3) take its place; each of their strikes heals them by half their power.",
    "When it dies, three Heads (2/3) take its place; each of their strikes heals them by half their power.",
    "Когда погибает, на его место встают две Головы (2/3); каждая их атака лечит их на половину АТК.",
    "Когда погибает, на его место встают три Головы (2/3); каждая их атака лечит их на половину АТК.",
  ],
  Archmage: () => [
    "At each turn end it gains 1 mana; at a turn end with 3+ mana, it spends 3 and strikes every enemy for its power.",
    "At each turn end it gains 1 mana; at a turn end with 3+ mana, it spends 3, strikes every enemy for its power and curses each one.",
    "В конце каждого хода получает 1 ману; в конце хода, если маны 3 или больше, тратит 3 и бьёт всех врагов на свою АТК.",
    "В конце каждого хода получает 1 ману; в конце хода, если маны 3 или больше, тратит 3, бьёт всех врагов на свою АТК и проклинает каждого.",
  ],
  Oracle: () => [
    "At the start of turn 3 and every turn after, it curses the enemy in front: −1 power.",
    "At the start of turn 3 and every turn after, it curses the enemies at the front and at the back: −1 power.",
    "В начале 3-го хода и каждого следующего проклинает переднего врага: −1 АТК.",
    "В начале 3-го хода и каждого следующего проклинает переднего и заднего врага: −1 АТК.",
  ],
  Berserker: () => [
    "Its strikes deal its power plus the HP it has lost.",
    "Its strikes deal its power plus the HP it has lost, and heal it by half its power.",
    "Его атаки наносят АТК плюс потерянные ОЗ.",
    "Его атаки наносят АТК плюс потерянные ОЗ и лечат его на половину АТК.",
  ],
};
