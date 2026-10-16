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
export const V2_SAY: Record<string, () => [string, string, string, string]> = {
  Candle: () => [
    "Instead of dying, it stays at 1 HP and loses 1 power. Once its power is 0, it dies.",
    "Instead of dying, it stays at 1 HP and loses 1 power, and each time it heals its neighbours by its power. Once its power is 0, it dies.",
    "Вместо смерти остаётся с 1 ОЗ и теряет 1 АТК. Когда его АТК доходит до 0, погибает.",
    "Вместо смерти остаётся с 1 ОЗ и теряет 1 АТК, и каждый раз лечит соседей на свою АТК. Когда его АТК доходит до 0, погибает.",
  ],
  Magnet: () => [
    "When Poison, Curse or Freeze is aimed at an ally, it lands on Magnet instead.",
    "When Poison, Curse or Freeze is aimed at an ally, it lands on Magnet instead, and Magnet gains 1 Shield each time.",
    "Когда в союзника летит Яд, Проклятие или Заморозка, они попадают в Магнит.",
    "Когда в союзника летит Яд, Проклятие или Заморозка, они попадают в Магнит, и он каждый раз получает 1 Щит.",
  ],
  Raven: () => [
    "The first time an ally dies in a battle, Raven uses that ally's rule once, right away.",
    "The first time an ally dies in a battle, Raven uses that ally's rule once, right away, and keeps it as a second rule of its own for the rest of the battle.",
    "Когда в бою впервые погибает союзник, Ворон сразу один раз использует его правило.",
    "Когда в бою впервые погибает союзник, Ворон сразу один раз использует его правило и до конца боя оставляет его себе как второе правило.",
  ],
  Seed: () => [
    "It does nothing on turns 1 and 2. From turn 3 on, at each turn start, it gains 2 HP and gives the ally behind it 1 Strength.",
    "It does nothing on turn 1. From turn 2 on, at each turn start, it gains 2 HP and gives the ally behind it 1 Strength.",
    "На 1-м и 2-м ходу ничего не делает. С 3-го хода в начале каждого хода получает 2 ОЗ и даёт союзнику позади 1 Силу.",
    "На 1-м ходу ничего не делает. Со 2-го хода в начале каждого хода получает 2 ОЗ и даёт союзнику позади 1 Силу.",
  ],
  "Lone Wolf": () => [
    "While it has 1 ally or none left, its strikes deal double.",
    "While it has 1 ally or none left, its strikes deal double and it strikes twice.",
    "Пока у него остался 1 союзник или ни одного, его атаки наносят двойной урон.",
    "Пока у него остался 1 союзник или ни одного, его атаки наносят двойной урон, и он бьёт дважды.",
  ],
  Drummer: () => [
    "When an ally strikes, it gains 1 Charge. At 3 Charge, it spends them and every ally behind it gains 1 Strength.",
    "When an ally strikes, it gains 1 Charge. At 3 Charge, it spends them and every ally gains 1 Strength.",
    "Когда союзник атакует, получает 1 Заряд. При 3 Зарядах тратит их, и все союзники позади получают 1 Силу.",
    "Когда союзник атакует, получает 1 Заряд. При 3 Зарядах тратит их, и все союзники получают 1 Силу.",
  ],
  Gravedigger: () => [
    "At each turn start, it hits the enemy in front for 1 per ally that has fallen this battle.",
    "At each turn start, it hits every enemy for 1 per ally that has fallen this battle.",
    "В начале каждого хода бьёт переднего врага на 1 за каждого союзника, павшего в этом бою.",
    "В начале каждого хода бьёт всех врагов на 1 за каждого союзника, павшего в этом бою.",
  ],
  Conductor: () => [
    "At the start of turns 3, 6, 9…, its neighbours use their rules again.",
    "At the start of turns 3, 6, 9…, its neighbours use their rules again and gain 1 Strength.",
    "В начале 3-го, 6-го, 9-го… хода его соседи используют свои правила ещё раз.",
    "В начале 3-го, 6-го, 9-го… хода его соседи используют свои правила ещё раз и получают 1 Силу.",
  ],
  "Coin Thief": () => [
    "At each turn start, it curses a random enemy (−1 power) and gains 1 Strength.",
    "At each turn start, it curses a random enemy (−1 power) and gains 1 Strength and 1 Shield.",
    "В начале каждого хода проклинает случайного врага (−1 АТК) и получает 1 Силу.",
    "В начале каждого хода проклинает случайного врага (−1 АТК) и получает 1 Силу и 1 Щит.",
  ],
  Lantern: () => [
    "At each turn start, if it has lost no HP, it gives the ally ahead of it 1 Shield.",
    "At each turn start, if it has lost no HP, it gives both its neighbours 1 Shield.",
    "В начале каждого хода, если не потерял ни одного ОЗ, даёт союзнику впереди 1 Щит.",
    "В начале каждого хода, если не потерял ни одного ОЗ, даёт обоим соседям 1 Щит.",
  ],
  "War Horn": () => [
    "At each turn start, if 2 or fewer of its allies are left (not counting itself), every ally gains 1 Strength.",
    "At each turn start, if 2 or fewer of its allies are left (not counting itself), every ally gains 1 Strength and 1 Shield.",
    "В начале каждого хода, если у него осталось не больше 2 союзников (не считая себя), все союзники получают 1 Силу.",
    "В начале каждого хода, если у него осталось не больше 2 союзников (не считая себя), все союзники получают 1 Силу и 1 Щит.",
  ],
  Vulture: () => [
    "When any unit falls, ally or enemy, it gains 1 Strength.",
    "When any unit falls, ally or enemy, it gains 1 Strength and heals by its power.",
    "Когда погибает любой боец, союзник или враг, получает 1 Силу.",
    "Когда погибает любой боец, союзник или враг, получает 1 Силу и лечится на свою АТК.",
  ],
  Hexer: () => [
    "At the start of turn 2, it silences the enemy at the back: that enemy loses its rule and statuses for the rest of the battle.",
    "At the start of turn 2, it silences the enemies at the back and at the front: they lose their rules and statuses for the rest of the battle.",
    "В начале 2-го хода заставляет замолчать заднего врага: тот до конца боя теряет правило и статусы.",
    "В начале 2-го хода заставляет замолчать заднего и переднего врага: те до конца боя теряют правила и статусы.",
  ],
  Hydra: () => [
    "When it dies, two Heads (2/3, no rule) take its place.",
    "When it dies, three Heads (2/3, no rule) take its place.",
    "Когда погибает, его место занимают две Головы (2/3, без правила).",
    "Когда погибает, его место занимают три Головы (2/3, без правила).",
  ],
  Archmage: () => [
    "At each turn end, it gains 1 Charge. At 3 Charge, it spends them and strikes every enemy for its power.",
    "At each turn end, it gains 1 Charge. At 3 Charge, it spends them, strikes every enemy for its power and curses each one (−1 power).",
    "В конце каждого хода получает 1 Заряд. При 3 Зарядах тратит их и бьёт всех врагов на свою АТК.",
    "В конце каждого хода получает 1 Заряд. При 3 Зарядах тратит их, бьёт всех врагов на свою АТК и проклинает каждого (−1 АТК).",
  ],
  Oracle: () => [
    "From turn 3 on, at each turn start, it curses the enemy in front (−1 power).",
    "From turn 3 on, at each turn start, it curses the enemies in front and at the back (−1 power).",
    "С 3-го хода в начале каждого хода проклинает переднего врага (−1 АТК).",
    "С 3-го хода в начале каждого хода проклинает переднего и заднего врага (−1 АТК).",
  ],
  Hourglass: () => [
    "At each turn end, it gains 1 Charge. At 3 Charge, it spends them and the ally behind it uses its rule again.",
    "At each turn end, it gains 1 Charge. At 3 Charge, it spends them and every ally behind it uses its rule again.",
    "В конце каждого хода получает 1 Заряд. При 3 Зарядах тратит их, и союзник позади использует своё правило ещё раз.",
    "В конце каждого хода получает 1 Заряд. При 3 Зарядах тратит их, и все союзники позади используют своё правило ещё раз.",
  ],
  Mirror: () => [
    "At battle start, copies the rule of the enemy in front of it.",
    "At battle start, copies the rule and the power of the enemy in front of it.",
    "В начале боя копирует правило врага перед собой.",
    "В начале боя копирует правило и силу врага перед собой.",
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
  Sniper: () => [
    "At the start of turns 2, 4, 6…, shoots the enemy at the back for its power.",
    "At the start of turns 2, 4, 6…, shoots the enemies at the back and at the front for its power.",
    "В начале 2-го, 4-го, 6-го… хода стреляет в заднего врага на свою АТК.",
    "В начале 2-го, 4-го, 6-го… хода стреляет в заднего и переднего врага на свою АТК.",
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
  Bodyguard: () => [
    "The first time each turn a hit is aimed at an ally, it takes that hit instead.",
    "The first time each turn a hit is aimed at an ally, it takes that hit instead and gives that ally 1 Shield.",
    "Первый за ход удар по союзнику принимает на себя.",
    "Первый за ход удар по союзнику принимает на себя и даёт этому союзнику 1 Щит.",
  ],
  Chrysalis: () => [
    "At the start of turn 3, it becomes a Butterfly (2/3) that strikes every enemy for its power at each turn start.",
    "At the start of turn 3, it becomes a Monarch (2/3) that, at each turn start, strikes every enemy and gives every ally 1 Strength.",
    "В начале 3-го хода превращается в Бабочку (2/3), которая в начале каждого хода бьёт всех врагов на свою АТК.",
    "В начале 3-го хода превращается в Монарха (2/3), который в начале каждого хода бьёт всех врагов и даёт всем союзникам 1 Силу.",
  ],
  Trickster: () => [
    "At battle start, it swaps rules with the enemy in front of it.",
    "At battle start, it takes the rule of the enemy in front of it and leaves it with none.",
    "В начале боя меняется правилами с врагом перед собой.",
    "В начале боя забирает правило врага перед собой и оставляет его без правила.",
  ],
  // batch 2 (#1047)
  Thornback: () => [
    "When it is hit, it pricks its attacker for 1.",
    "When it is hit, it pricks its attacker for its power.",
    "Когда его бьют, колет обидчика на 1.",
    "Когда его бьют, колет обидчика на свою АТК.",
  ],
  Hornet: () => [
    "Each of its strikes poisons the enemy it hits.",
    "Each of its strikes poisons every enemy.",
    "Каждая его атака отравляет того, кого он бьёт.",
    "Каждая его атака отравляет всех врагов.",
  ],
  Prism: () => [
    "When an enemy gives it Poison, Curse or Freeze, that enemy gets the same amount too.",
    "When an enemy gives it Poison, Curse or Freeze, it reflects it fully: the enemy gets it and Prism doesn't.",
    "Когда враг даёт ей Яд, Проклятие или Заморозку, этот враг получает столько же.",
    "Когда враг даёт ей Яд, Проклятие или Заморозку, она отражает их целиком: враг получает их, а Призма нет.",
  ],
  "Fortune Teller": () => [
    "At battle start, if the enemy has more units than your side, every ally gains 1 Shield; otherwise every ally gains 1 Strength.",
    "At battle start, if the enemy has more units than your side, every ally gains 2 Shield; otherwise every ally gains 2 Strength.",
    "В начале боя, если у врага больше бойцов, чем у вас, все союзники получают 1 Щит; иначе все союзники получают 1 Силу.",
    "В начале боя, если у врага больше бойцов, чем у вас, все союзники получают 2 Щита; иначе все союзники получают 2 Силы.",
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
  Berserker: () => [
    "Its strikes deal its power plus the HP it has lost.",
    "Its strikes deal its power plus the HP it has lost, and heal it by half its power.",
    "Его атаки наносят АТК плюс потерянные ОЗ.",
    "Его атаки наносят АТК плюс потерянные ОЗ и лечат его на половину АТК.",
  ],
  // the rethink (#1063)
  Frostling: () => [
    "Each of its strikes gives the enemy it hits 1 Freeze (it skips its next strike).",
    "Each of its strikes gives the enemy it hits 2 Freeze (it skips its next two strikes).",
    "Каждая его атака даёт врагу, которого он бьёт, 1 Заморозку (тот пропускает свою следующую атаку).",
    "Каждая его атака даёт врагу, которого он бьёт, 2 Заморозки (тот пропускает свои следующие две атаки).",
  ],
  Priest: () => [
    "At battle start, it gives the ally ahead of it Blessing equal to its power. The next time that ally would die, it lives on with that much HP instead.",
    "At battle start, it gives both its neighbours Blessing equal to its power. The next time one of them would die, it lives on with that much HP instead.",
    "В начале боя даёт союзнику впереди Благословение, равное своей АТК. Когда тот в следующий раз должен погибнуть, он вместо этого остаётся жить с таким ОЗ.",
    "В начале боя даёт обоим соседям Благословение, равное своей АТК. Когда один из них в следующий раз должен погибнуть, он вместо этого остаётся жить с таким ОЗ.",
  ],
  Battery: () => [
    "When an ally is hit, it gains 1 Charge. At 3 Charge, it spends them and every ally gains 1 Shield.",
    "When an ally is hit, it gains 1 Charge. At 3 Charge, it spends them and every ally gains 1 Shield and heals 1 HP.",
    "Когда союзника бьют, получает 1 Заряд. При 3 Зарядах тратит их, и все союзники получают 1 Щит.",
    "Когда союзника бьют, получает 1 Заряд. При 3 Зарядах тратит их, и все союзники получают 1 Щит и лечатся на 1 ОЗ.",
  ],
  "Frost Witch": () => [
    "At the start of turn 2, every enemy gets 1 Freeze.",
    "At the start of turn 2, every enemy gets 1 Freeze, and the enemy in front gets 1 more.",
    "В начале 2-го хода все враги получают 1 Заморозку.",
    "В начале 2-го хода все враги получают 1 Заморозку, а передний враг — ещё 1.",
  ],
  Bully: () => [
    "Its strikes deal double to an enemy that has Poison, Curse or Freeze.",
    "Its strikes deal double to an enemy that has Poison, Curse or Freeze, and each such strike heals it by its power.",
    "Его атаки наносят двойной урон врагу с Ядом, Проклятием или Заморозкой.",
    "Его атаки наносят двойной урон врагу с Ядом, Проклятием или Заморозкой, и каждая такая атака лечит его на его силу.",
  ],
  Alchemist: () => [
    "At each turn end, every enemy that has Poison gets 1 more Poison.",
    "At each turn end, every enemy that has Poison gets 2 more Poison.",
    "В конце каждого хода каждый враг с Ядом получает ещё 1 Яд.",
    "В конце каждого хода каждый враг с Ядом получает ещё 2 Яда.",
  ],
  Executioner: () => [
    "When its strike kills an enemy, it strikes again.",
    "When its strike kills an enemy, it strikes again and gains 1 Strength.",
    "Когда его атака убивает врага, он бьёт ещё раз.",
    "Когда его атака убивает врага, он бьёт ещё раз и получает 1 Силу.",
  ],
  // batch 3 (#1048)
  "Plague Rat": () => [
    "When it dies, every enemy gains 1 Poison.",
    "When it dies, every enemy gains 2 Poison.",
    "Когда погибает, все враги получают 1 Яд.",
    "Когда погибает, все враги получают 2 Яда.",
  ],
  Martyr: () => [
    "When it dies, every ally gains Strength equal to half its power (rounded down).",
    "When it dies, every ally gains Strength equal to its power.",
    "Когда погибает, все союзники получают Силу, равную половине его АТК (с округлением вниз).",
    "Когда погибает, все союзники получают Силу, равную его АТК.",
  ],
  Ox: () => [
    "Its strikes deal damage equal to its current HP instead of its power.",
    "Its strikes deal damage equal to its current HP plus its power.",
    "Его атаки наносят урон, равный его текущим ОЗ, а не АТК.",
    "Его атаки наносят урон, равный его текущим ОЗ плюс его АТК.",
  ],
  Daredevil: () => [
    "While it has lost no HP, its strikes deal triple.",
    "While it has lost no HP, its strikes deal four times its power.",
    "Пока он не потерял ни одного ОЗ, его атаки наносят тройной урон.",
    "Пока он не потерял ни одного ОЗ, его атаки наносят урон в четыре раза больше его АТК.",
  ],
  Duelist: () => [
    "Its strikes deal damage equal to the struck enemy's power instead of its own.",
    "Its strikes deal damage equal to the struck enemy's power plus its own.",
    "Его атаки наносят урон, равный АТК врага, которого он бьёт, а не своей.",
    "Его атаки наносят урон, равный АТК врага, которого он бьёт, плюс своей.",
  ],
  Paladin: () => [
    "Its strikes deal its power plus 1 per Shield it holds.",
    "Its strikes deal its power plus 1 per Shield it holds. It starts the battle with 2 Shield.",
    "Его атаки наносят урон, равный его АТК плюс 1 за каждый его Щит.",
    "Его атаки наносят урон, равный его АТК плюс 1 за каждый его Щит. Начинает бой с 2 Щитами.",
  ],
  Squire: () => [
    "When an ally gains Shield, it gains 1 Strength.",
    "When an ally gains Shield, it and that ally each gain 1 Strength.",
    "Когда союзник получает Щит, получает 1 Силу.",
    "Когда союзник получает Щит, он и этот союзник получают по 1 Силе.",
  ],
  Dynamo: () => [
    "When it strikes, it gains 1 Charge. At 3 Charge, it spends them and strikes the same enemy again.",
    "When it strikes, it gains 1 Charge. At 3 Charge, it spends them, strikes the same enemy again and gains 1 Strength.",
    "Когда атакует, получает 1 Заряд. При 3 Зарядах тратит их и бьёт того же врага ещё раз.",
    "Когда атакует, получает 1 Заряд. При 3 Зарядах тратит их, бьёт того же врага ещё раз и получает 1 Силу.",
  ],
  Avenger: () => [
    "When an ally dies, it hits the enemy in front for that ally's power.",
    "When an ally dies, it hits the enemy in front for that ally's power and gains 1 Strength.",
    "Когда союзник погибает, бьёт переднего врага на АТК павшего союзника.",
    "Когда союзник погибает, бьёт переднего врага на АТК павшего союзника и получает 1 Силу.",
  ],
  Siphon: () => [
    "When an enemy would heal, Siphon gets the heal instead.",
    "When an enemy would heal, Siphon gets the heal instead and gains 1 Strength.",
    "Когда враг должен вылечиться, лечение получает Сифон.",
    "Когда враг должен вылечиться, лечение получает Сифон, и он получает 1 Силу.",
  ],
  Comet: () => [
    "At the start of turns 4, 8, 12…, it hits every enemy for the turn number (4 on turn 4, 8 on turn 8…).",
    "At the start of turns 3, 6, 9…, it hits every enemy for the turn number (3 on turn 3, 6 on turn 6…).",
    "В начале 4-го, 8-го, 12-го… хода бьёт всех врагов на номер хода (4 на 4-м ходу, 8 на 8-м…).",
    "В начале 3-го, 6-го, 9-го… хода бьёт всех врагов на номер хода (3 на 3-м ходу, 6 на 6-м…).",
  ],
  Reaper: () => [
    "At each turn start, if only 1 enemy is left, it hits that enemy for its power.",
    "At each turn start, if 2 or fewer enemies are left, it hits every enemy for its power.",
    "В начале каждого хода, если остался только 1 враг, бьёт его на свою АТК.",
    "В начале каждого хода, если осталось не больше 2 врагов, бьёт всех врагов на свою АТК.",
  ],
  Heckler: () => [
    "When the enemy in front uses its rule, it hits that enemy for its power.",
    "When the enemy in front uses its rule, it hits that enemy for its power and curses it (−1 power).",
    "Когда передний враг использует своё правило, бьёт его на свою АТК.",
    "Когда передний враг использует своё правило, бьёт его на свою АТК и проклинает (−1 АТК).",
  ],
  "Witch Doctor": () => [
    "When an enemy gains Poison, Curse or Freeze, it gains 1 Charge. At 3 Charge, it spends them and heals every ally for its power.",
    "When an enemy gains Poison, Curse or Freeze, it gains 1 Charge. At 3 Charge, it spends them and heals every ally for its power and gives each 1 Shield.",
    "Когда враг получает Яд, Проклятие или Заморозку, получает 1 Заряд. При 3 Зарядах тратит их и лечит всех союзников на свою АТК.",
    "Когда враг получает Яд, Проклятие или Заморозку, получает 1 Заряд. При 3 Зарядах тратит их, лечит всех союзников на свою АТК и даёт каждому 1 Щит.",
  ],
  Broodmother: () => [
    "At the start of turns 2, 4, 6…, a Spiderling (1/1) joins the front of its line. Each strike of a Spiderling gives the struck enemy 1 Poison.",
    "At each turn start, a Spiderling (1/1) joins the front of its line. Each strike of a Spiderling gives the struck enemy 1 Poison.",
    "В начале 2-го, 4-го, 6-го… хода в начало его ряда встаёт Паучок (1/1). Каждая атака Паучка даёт врагу 1 Яд.",
    "В начале каждого хода в начало его ряда встаёт Паучок (1/1). Каждая атака Паучка даёт врагу 1 Яд.",
  ],
  Dragon: () => [
    "When it strikes, every enemy also takes half its power (rounded down).",
    "When it strikes, every enemy also takes its full power.",
    "Когда атакует, все враги также получают урон в половину его АТК (с округлением вниз).",
    "Когда атакует, все враги также получают урон, равный его АТК.",
  ],
  Kraken: () => [
    "At each turn end, it hits the enemy in front for 1 per hit it has taken this battle.",
    "At each turn end, it hits every enemy for 1 per hit it has taken this battle.",
    "В конце каждого хода бьёт переднего врага на 1 за каждый удар, полученный им в этом бою.",
    "В конце каждого хода бьёт всех врагов на 1 за каждый удар, полученный им в этом бою.",
  ],
  Puppeteer: () => [
    "On turns 2, 4, 6…, the first time an enemy strikes, the strike hits the enemy at the back instead.",
    "On turns 2, 4, 6…, the first time an enemy strikes, the strike hits the enemy at the back instead, and it gains 1 Shield.",
    "На 2-м, 4-м, 6-м… ходу, когда враг атакует впервые за ход, удар попадает в заднего врага.",
    "На 2-м, 4-м, 6-м… ходу, когда враг атакует впервые за ход, удар попадает в заднего врага, а он получает 1 Щит.",
  ],
  Phoenix: () => [
    "The first time it dies, it comes back at the back of its line with HP equal to its power.",
    "The first time it dies, it comes back at the back of its line with HP equal to its power and hits every enemy for its power.",
    "Когда погибает впервые, возвращается в конец своего ряда с ОЗ, равными его АТК.",
    "Когда погибает впервые, возвращается в конец своего ряда с ОЗ, равными его АТК, и бьёт всех врагов на свою АТК.",
  ],
  Volcano: () => [
    "At the start of turns 2, 4, 6…, it hits every unit on the field for 1, its own side and itself included.",
    "At the start of turns 2, 4, 6…, it hits every enemy for 1 and every ally gains 1 Shield.",
    "В начале 2-го, 4-го, 6-го… хода бьёт всех бойцов на поле на 1, включая свою сторону и себя.",
    "В начале 2-го, 4-го, 6-го… хода бьёт всех врагов на 1, а все союзники получают 1 Щит.",
  ],
};
