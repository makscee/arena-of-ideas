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
    "Each time it is hit, it gains 1 Shield.",
    "Each time it is hit, it gains 2 Shield.",
    "\u041a\u0430\u0436\u0434\u044b\u0439 \u0440\u0430\u0437, \u043a\u043e\u0433\u0434\u0430 \u0435\u0433\u043e \u0431\u044c\u044e\u0442, \u043f\u043e\u043b\u0443\u0447\u0430\u0435\u0442 1 \u0429\u0438\u0442.",
    "\u041a\u0430\u0436\u0434\u044b\u0439 \u0440\u0430\u0437, \u043a\u043e\u0433\u0434\u0430 \u0435\u0433\u043e \u0431\u044c\u044e\u0442, \u043f\u043e\u043b\u0443\u0447\u0430\u0435\u0442 2 \u0429\u0438\u0442\u0430.",
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
    "At each turn start, hits the enemy in front for 1 per fallen ally.",
    "At each turn start, hits every enemy for 1 per fallen ally.",
    "\u0412 \u043d\u0430\u0447\u0430\u043b\u0435 \u043a\u0430\u0436\u0434\u043e\u0433\u043e \u0445\u043e\u0434\u0430 \u0431\u044c\u0451\u0442 \u043f\u0435\u0440\u0435\u0434\u043d\u0435\u0433\u043e \u0432\u0440\u0430\u0433\u0430 \u043d\u0430 1 \u0437\u0430 \u043a\u0430\u0436\u0434\u043e\u0433\u043e \u043f\u0430\u0432\u0448\u0435\u0433\u043e \u0441\u043e\u044e\u0437\u043d\u0438\u043a\u0430.",
    "\u0412 \u043d\u0430\u0447\u0430\u043b\u0435 \u043a\u0430\u0436\u0434\u043e\u0433\u043e \u0445\u043e\u0434\u0430 \u0431\u044c\u0451\u0442 \u0432\u0441\u0435\u0445 \u0432\u0440\u0430\u0433\u043e\u0432 \u043d\u0430 1 \u0437\u0430 \u043a\u0430\u0436\u0434\u043e\u0433\u043e \u043f\u0430\u0432\u0448\u0435\u0433\u043e \u0441\u043e\u044e\u0437\u043d\u0438\u043a\u0430.",
  ],
  Conductor: () => [
    "Every second turn, its neighbours act again.",
    "Every second turn, its neighbours act again and gain 1 Strength.",
    "Каждый второй ход его соседи действуют ещё раз.",
    "Каждый второй ход его соседи действуют ещё раз и получают 1 Силу.",
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
    "\u0412 \u043d\u0430\u0447\u0430\u043b\u0435 \u043a\u0430\u0436\u0434\u043e\u0433\u043e \u0445\u043e\u0434\u0430 \u0441 \u0432\u0435\u0440\u043e\u044f\u0442\u043d\u043e\u0441\u0442\u044c\u044e 50% \u0433\u0440\u0430\u0431\u0438\u0442 \u0441\u043b\u0443\u0447\u0430\u0439\u043d\u043e\u0433\u043e \u0432\u0440\u0430\u0433\u0430 \u043d\u0430 \u0441\u0432\u043e\u044e \u0410\u0422\u041a.",
    "\u0412 \u043d\u0430\u0447\u0430\u043b\u0435 \u043a\u0430\u0436\u0434\u043e\u0433\u043e \u0445\u043e\u0434\u0430 \u0441 \u0432\u0435\u0440\u043e\u044f\u0442\u043d\u043e\u0441\u0442\u044c\u044e 75% \u0433\u0440\u0430\u0431\u0438\u0442 \u0441\u043b\u0443\u0447\u0430\u0439\u043d\u043e\u0433\u043e \u0432\u0440\u0430\u0433\u0430 \u043d\u0430 \u0441\u0432\u043e\u044e \u0410\u0422\u041a.",
  ],
  Scavenger: () => [
    "When an ally falls, it gains 1 Strength.",
    "When an ally falls, it gains 1 Strength and heals by its power.",
    "Когда союзник падает, получает 1 Силу.",
    "Когда союзник падает, получает 1 Силу и лечится на свою АТК.",
  ],
  Lantern: () => [
    "At each turn start, if it has not been hurt, it gives the ally ahead of it 1 Shield.",
    "At each turn start, if it has not been hurt, it gives both its neighbours 1 Shield.",
    "В начале каждого хода, если его ещё не ранили, даёт союзнику впереди 1 Щит.",
    "В начале каждого хода, если его ещё не ранили, даёт обоим соседям 1 Щит.",
  ],
  Hornet: () => [
    "Each of its strikes poisons the enemy it hits.",
    "Each of its strikes poisons every enemy.",
    "Каждая его атака отравляет того, кого он бьёт.",
    "Каждая его атака отравляет всех врагов.",
  ],
  "Echo Bell": () => [
    "At battle start, it learns the rule of the ally behind it; every second turn, it rings that rule once.",
    "At battle start, it learns the rule of the ally behind it; every turn, it rings that rule once.",
    "В начале боя запоминает правило союзника позади; каждый второй ход исполняет его.",
    "В начале боя запоминает правило союзника позади; каждый ход исполняет его.",
  ],
  Prism: () => [
    "A random hit aimed at it bends onto a random enemy instead.",
    "A random hit aimed at it bends onto a random enemy instead, and it gains 1 Shield.",
    "Случайный удар по нему вместо этого уходит в случайного врага.",
    "Случайный удар по нему вместо этого уходит в случайного врага, и он получает 1 Щит.",
  ],
  "Wolf Pup": () => [
    "When the first ally falls, it grows into a Dire Wolf (4/10) that gains 1 Strength at each turn end.",
    "When the first ally falls, it grows into a Dire Wolf (5/12) that gains 1 Strength at each turn end.",
    "\u041a\u043e\u0433\u0434\u0430 \u043f\u0430\u0434\u0430\u0435\u0442 \u043f\u0435\u0440\u0432\u044b\u0439 \u0441\u043e\u044e\u0437\u043d\u0438\u043a, \u0432\u044b\u0440\u0430\u0441\u0442\u0430\u0435\u0442 \u0432 \u041b\u044e\u0442\u043e\u0432\u043e\u043b\u043a\u0430 (4/10), \u043a\u043e\u0442\u043e\u0440\u044b\u0439 \u0432 \u043a\u043e\u043d\u0446\u0435 \u043a\u0430\u0436\u0434\u043e\u0433\u043e \u0445\u043e\u0434\u0430 \u043f\u043e\u043b\u0443\u0447\u0430\u0435\u0442 1 \u0421\u0438\u043b\u0443.",
    "\u041a\u043e\u0433\u0434\u0430 \u043f\u0430\u0434\u0430\u0435\u0442 \u043f\u0435\u0440\u0432\u044b\u0439 \u0441\u043e\u044e\u0437\u043d\u0438\u043a, \u0432\u044b\u0440\u0430\u0441\u0442\u0430\u0435\u0442 \u0432 \u041b\u044e\u0442\u043e\u0432\u043e\u043b\u043a\u0430 (5/12), \u043a\u043e\u0442\u043e\u0440\u044b\u0439 \u0432 \u043a\u043e\u043d\u0446\u0435 \u043a\u0430\u0436\u0434\u043e\u0433\u043e \u0445\u043e\u0434\u0430 \u043f\u043e\u043b\u0443\u0447\u0430\u0435\u0442 1 \u0421\u0438\u043b\u0443.",
  ],
  "War Horn": () => [
    "At each turn start, with 3 or fewer allies standing, it gives every ally 1 Strength.",
    "At each turn start, with 3 or fewer allies standing, it gives every ally 1 Strength and 1 Shield.",
    "\u0412 \u043d\u0430\u0447\u0430\u043b\u0435 \u043a\u0430\u0436\u0434\u043e\u0433\u043e \u0445\u043e\u0434\u0430, \u0435\u0441\u043b\u0438 \u0441\u043e\u044e\u0437\u043d\u0438\u043a\u043e\u0432 \u043e\u0441\u0442\u0430\u043b\u043e\u0441\u044c 3 \u0438\u043b\u0438 \u043c\u0435\u043d\u044c\u0448\u0435, \u0434\u0430\u0451\u0442 \u0432\u0441\u0435\u043c \u0441\u043e\u044e\u0437\u043d\u0438\u043a\u0430\u043c 1 \u0421\u0438\u043b\u0443.",
    "\u0412 \u043d\u0430\u0447\u0430\u043b\u0435 \u043a\u0430\u0436\u0434\u043e\u0433\u043e \u0445\u043e\u0434\u0430, \u0435\u0441\u043b\u0438 \u0441\u043e\u044e\u0437\u043d\u0438\u043a\u043e\u0432 \u043e\u0441\u0442\u0430\u043b\u043e\u0441\u044c 3 \u0438\u043b\u0438 \u043c\u0435\u043d\u044c\u0448\u0435, \u0434\u0430\u0451\u0442 \u0432\u0441\u0435\u043c \u0441\u043e\u044e\u0437\u043d\u0438\u043a\u0430\u043c 1 \u0421\u0438\u043b\u0443 \u0438 1 \u0429\u0438\u0442.",
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
    "\u0412 \u043d\u0430\u0447\u0430\u043b\u0435 2-\u0433\u043e \u0445\u043e\u0434\u0430 \u0437\u0430\u0433\u043b\u0443\u0448\u0430\u0435\u0442 \u0437\u0430\u0434\u043d\u0435\u0433\u043e \u0432\u0440\u0430\u0433\u0430: \u0442\u043e\u0442 \u0442\u0435\u0440\u044f\u0435\u0442 \u043f\u0440\u0430\u0432\u0438\u043b\u043e \u0438 \u0441\u0442\u0430\u0442\u0443\u0441\u044b.",
    "\u0412 \u043d\u0430\u0447\u0430\u043b\u0435 2-\u0433\u043e \u0445\u043e\u0434\u0430 \u0437\u0430\u0433\u043b\u0443\u0448\u0430\u0435\u0442 \u0437\u0430\u0434\u043d\u0435\u0433\u043e \u0438 \u043f\u0435\u0440\u0435\u0434\u043d\u0435\u0433\u043e \u0432\u0440\u0430\u0433\u0430: \u043e\u043d\u0438 \u0442\u0435\u0440\u044f\u044e\u0442 \u043f\u0440\u0430\u0432\u0438\u043b\u0430 \u0438 \u0441\u0442\u0430\u0442\u0443\u0441\u044b.",
  ],
  "Fortune Teller": () => [
    "At battle start, flips a coin: heads, every ally gains 2 Strength; tails, every ally gains 2 Shield.",
    "At battle start, flips a coin: heads, every ally gains 3 Strength; tails, every ally gains 3 Shield.",
    "В начале боя бросает монету: орёл — все союзники получают 2 Силы, решка — 2 Щита.",
    "В начале боя бросает монету: орёл — все союзники получают 3 Силы, решка — 3 Щита.",
  ],
  Aegis: () => [
    "The first hit on it each turn does nothing.",
    "The first hit on it each turn does nothing, and the ally behind it gains 1 Shield.",
    "Первый удар по нему за ход ничего не делает.",
    "Первый удар по нему за ход ничего не делает, а союзник позади получает 1 Щит.",
  ],
  Gravecaller: () => [
    "When the first ally falls, it calls that ally back with 1 HP.",
    "When the first ally falls, it calls that ally back with HP equal to its power.",
    "Когда падает первый союзник, возвращает его с 1 ОЗ.",
    "Когда падает первый союзник, возвращает его с ОЗ, равными своей АТК.",
  ],
  Jester: () => [
    "The first time it is hit, it trades rules with its attacker.",
    "The first time it is hit, it trades rules with its attacker and gains 1 Shield.",
    "Когда его бьют впервые, меняется правилами с обидчиком.",
    "Когда его бьют впервые, меняется правилами с обидчиком и получает 1 Щит.",
  ],
  Hydra: () => [
    "When it dies, two Heads (2/3) take its place; each of their strikes heals them by half their power.",
    "When it dies, three Heads (2/3) take its place; each of their strikes heals them by half their power.",
    "Когда погибает, на его место встают две Головы (2/3); каждая их атака лечит их на половину АТК.",
    "Когда погибает, на его место встают три Головы (2/3); каждая их атака лечит их на половину АТК.",
  ],
  Archmage: () => [
    "At each turn end it gathers a mana; at 3 mana, it strikes every enemy for its power.",
    "At each turn end it gathers a mana; at 3 mana, it strikes every enemy for its power and curses each one.",
    "В конце каждого хода копит ману; на 3 мане бьёт всех врагов на свою АТК.",
    "В конце каждого хода копит ману; на 3 мане бьёт всех врагов на свою АТК и проклинает каждого.",
  ],
  Oracle: () => [
    "From the third turn on, at each turn start, it curses the enemy in front: −1 power.",
    "From the third turn on, at each turn start, it curses every enemy: −1 power.",
    "С третьего хода, в начале каждого хода, проклинает переднего врага: −1 АТК.",
    "С третьего хода, в начале каждого хода, проклинает всех врагов: −1 АТК.",
  ],
  Berserker: () => [
    "Its strikes deal its power plus the HP it has lost.",
    "Its strikes deal its power plus the HP it has lost, and heal it by half its power.",
    "Его атаки наносят АТК плюс потерянные ОЗ.",
    "Его атаки наносят АТК плюс потерянные ОЗ и лечат его на половину АТК.",
  ],
};
