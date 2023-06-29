First Build @content
    @abilities
        [Vitality] "+X" HP +Medics
        [Mend] Heal X DMG +Medics
        [Shield] Ignore next taken damage +Paladins
        [Martyr] After death, give [Blessing](X) to all allies +Paladins
        [Blessing] "+X/+X" +Paladins
        [Strength] "+X" ATK +Warriors
        [Defense] Decrease incoming damage by X +Warriors
        [Weakness] "-X" ATK +Witches
        [Decay] reduce HP by X +Witches
        [Thorns] Deal X damage to attacker +Druids
        [Rebirth] After death, revive with X HP +Druids
        [Volatility] Deal X damage to all enemies after death +Elementals
        [Splash] Deal X damage to all enemies after strike +Elementals
        [Marked] Taken damage increased by X +Hunters
        [Shoot] Deal ATK damage X times to random enemy +Hunters
        [MagicMissile] Deal X damage to random enemy +Wizards
    @heroes
        [Vitality]
            Lifebringer - Battle Start: gain [Vitality] for each unit in battle {cm:2023-06-29T13:07:12}
            Endurance Knight - Turn End: gain [Vitality] {cm:2023-06-29T13:13:32}
            Vitality Giver - Turn End: give [Vitality] to random ally {cm:2023-06-29T12:56:42}
            Vital Reclaimer - After strike: give [Vitality] to all allies {cm:2023-06-28T13:00:01}
            Genesis Warden - Battle Start: give [Vitality](2) to all allies {cm:2023-06-28T12:48:50}
            Last Breath - Before death: give [Vitality](2) to all allies
            Lifebloom - [Vitality] gives 1 more HP per charge
            Vital Striker - On kill: give [Vitality](3) to all allies
        [Mend]
            Healing Hand - Turn End: [Mend](3) injured ally
            Mourning Spirit - Ally died: [Mend](5) all allies
            Recovery Priest - Turn End: [Mend] all allies
            Lifetide Warrior - After strike: [Mend](2) self
            Overhealer - [Mend] heals 1 more DMG per charge
            Life Reclaimer - On kill: [Mend](5) all allies
        [Shield]
            Bulwark Champion - Battle Start: gain [Shield]
            Safeguard - Before death: give [Shield] to ally behind
            Resilience Warden - Ally died: gain [Shield]
            Aegis Bearer - Battle Start: give [Shield] to allies with 10+ HP
            Reflective Guardian - [Shield] reflects damage back to attacker
            Shieldbearer - Battle Start: give [Shield] to adjacent allies
            Shield Slinger - On kill: gain [Shield]
        [Martyr]
            Sacrificial Soul - Battle Start: gain [Martyr] for each enemy
            Undying Martyr - Ally died: gain [Martyr]
            Searing Sacrifice - Before strike: gain [Martyr]
            Killblessed - On kill: gain [Martyr](2)
        [Strength]
            Empowerment Warrior - On kill: give [Strength] to all allies
            Warcry Barbarian - Battle Start: give [Strength] to all allies
            Soldier - Before death: give [Strength](4) to random ally
            Commander - Ally died: give [Strength] to ally behind
        [Defense]
            Stalwart Defender - Battle Start: gain [Defense](99)
            Shieldsmith - After strike: give [Defense] to random ally
            Fortress Master - Battle Start: gain [Defense] per empty slot
            Deflective Knight - On kill: give [Defense] to all allies
        [Weakness]
            Cursed Seer - Ally died: apply [Weakness] to killer
            Harbinger of Despair - Before death: apply [Weakness] to all enemies
            Potion Thrower - Battle Start: apply [Weakness] to 3 enemies
            Drainspirit Witch - [Weakness] also reduces HP
        [Thorns]
            Bramble Knight - Battle Start: apply [Thorns] to right ally
            Thornwall Sentinel - Battle Start: gain [Thorns] for each enemy
            Thorned Protector - [Thorns] deal 1 more DMG per charge
            Thornwreath - Ally died: give [Thorns] to all allies
        [Rebirth]
            Phoenix Mage - Battle Start: gain [Rebirth]
            Soulclaimer - On kill: gain [Rebirth]
            Rebirth Ritualist - Battle Start: give [Rebirth] to first ally
        [Volatility]
            Firestorm Warlock - Battle Start: gain [Volatility](2)
            Volatile Slayer - On kill: gain [Volatility]
            Explosion Bringer - Before death: give [Volatility](3) to left ally
        [Splash]
            Rainmaker - Battle Start: gain [Splash]
            Wavecrusher - On kill: gain [Splash]
            Splashtide Warrior - Battle Start: gain [Splash] for each enemy
        [Marked]
            Marking Hand - Before strike: apply [Marked]
            Sheriff - Battle Start: apply [Marked] to all enemies
            Dark Sigil - On kill: apply [Marked](3) to next enemy
        [Shoot]
            Archon Marksman - Before strike: [Shoot]
            Avenger - Ally died: [Shoot]
            Last Arrow - Last ally died: [Shoot](3)
            Sharpshooter - Battle Start: [Shoot](2)
        [MagicMissile]
            Arcane Gunner - Battle Start: use [MagicMissile](4)
            Magic Vendetta - Ally died: use [MagicMissile](2)
            Arcane Punisher - On kill: use [MagicMissile](6)
    @enemies
        Havoc - After Death, deal 1 DMG to all enemies
        Hexer - After Death, apply [Weakness] to killer
        Vengeance - After Death, deal 2 DMG to killer
        Sacrilege - Has [Martyr]
        Wither - After death: apply [Decay] to all enemies
        Snake - Kill after dealing damage
        Leech - Enemy died: gain [Vitality](2)
        Fiend  - Enemy died: gain [Strength]
        Gorge - Turn End: gain [Vitality]
        Bane - Ally died: apply [Weakness] to random enemy

