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
            Start of battle: gain [Vitality] for each unit in battle
            End of turn: gain [Vitality]
            End of turn: give [Vitality] to random ally
            After strike: give [Vitality] to all allies
            Start of battle: give [Vitality](2) to all allies
            Before death: give [Vitality](2) to all allies
            [Vitality] gives 1 more HP per charge
            On kill: give [Vitality](3) to all allies
        [Mend]
            End of turn: [Mend](3) injured ally
            Ally died: [Mend](5) all allies
            End of turn: [Mend] all allies
            After strike: [Mend](2) self
            [Mend] heals 1 more DMG per charge
            On kill: [Mend](5) all allies
        [Shield]
            Start of battle: gain [Shield]
            Before death: give [Shield] to ally behind
            Ally died: gain [Shield]
            Start of battle: give [Shield] to allies with 10+ HP
            [Shield] reflects damage back to attacker
            Start of battle: give [Shield] to adjacent allies
            On kill: gain [Shield]
        [Martyr]
            Start of battle: gain [Martyr] for each enemy
            Ally died: gain [Martyr]
            Before strike: gain [Martyr]
            On kill: gain [Martyr](2)
        [Strength]
            On kill: give [Strength] to all allies
            Start of battle: give [Strength] to all allies
            Before death: give [Strength](4) to random ally
            Ally died: give [Strength] to ally behind
        [Defense]
            Start of battle: gain [Defense](99)
            After strike: give [Defense] to random ally
            Start of battle: gain [Defense] per empty slot
            On kill: give [Defense] to all allies
        [Weakness]
            Ally died: apply [Weakness] to killer
            Before death: apply [Weakness] to all enemies
            Start of battle: apply [Weakness] to 3 enemies
            [Weakness] also reduces HP
        [Thorns]
            Start of battle: apply [Thorns] to right ally
            Start of battle: gain [Thorns] for each enemy
            [Thorns] deal 1 more DMG per charge
            Ally died: give [Thorns] to all allies
        [Rebirth]
            Start of battle: gain [Rebirth]
            On kill: gain [Rebirth]
            Start of battle: give [Rebirth] to first ally
        [Volatility]
            Start of battle: gain [Volatility](2)
            On kill: gain [Volatility]
            Before death: give [Volatility](3) to left ally
        [Splash]
            Start of battle: gain [Splash]
            On kill: gain [Splash]
            Start of battle: gain [Splash] for each enemy
        [Marked]
            Before strike: apply [Marked]
            Start of battle: apply [Marked] to all enemies
            On kill: apply [Marked](3) to next enemy
        [Shoot]
            Before strike: [Shoot]
            Ally died: [Shoot]
            Last ally died: [Shoot](3)
            Start of battle: [Shoot](2)
        [MagicMissile]
            Start of battle: use [MagicMissile](4)
            Ally died: use [MagicMissile](2)
            On kill: use [MagicMissile](6)
    @enemies
        After Death, deal 1 DMG to all enemies
        After Death, apply [Weakness] to killer
        After Death, deal 2 DMG to killer
        Has [Martyr]
        After death: apply [Decay] to all enemies
        Kill after dealing damage
        Enemy died: gain [Vitality](2)
        Enemy died: gain [Strength]
        End of turn: gain [Vitality]
        Ally died: apply [Weakness] to random enemy
