@content +Elementals {c}
    [Splash]: "+1" damage to all enemies after strike once
    Torrent [2/1]: Battle start: give [Splash] to adjacent allies.
    Annihilator [1/2]: Enemy killed: gain [Splash].
    Surge [2/1]: Turn start: gain [Splash].
    Impact [2/1]: [Splash] "+1" DMG.
    Echo [1/2]: After death: give [Splash] (2) to all allies.
    Surgeon [2/1]: Turn end: give [Splash] to random ally.
Next Iteration
    @abilities
        [Vitality]: "+1" HP +Medics {cm:2023-05-17T16:13:18}
        [Mend]: Heal 1 damage +Medics {cm:2023-05-17T16:13:17}
        [Blessing]: Gain \+1/+1 +Medics
        [Martyr]: After Death, give [Blessing] to random ally +Medics
        [Might]: "+1" ATK +Warriors 
        [Endurance]: "+2" HP, but "-1" ATK +Warriors 
        [Honored]: After Death: give [Might] to all allies +Warriors 
        [Weakness]: Lose 1 ATK +Witches 
        [Cursed]: After Death, apply [Weakness] to killer +Witches 
        [Marked]: Next taken damage is doubled +Witches 
        [Enrage]: "+2" ATK, take 1 damage +Orcs {cm:2023-05-18T16:29:36}
        [Fury]: Gain "+1" ATK when damage is taken +Orcs 
        [Adaptation]: "+1/+1", until the end of the turn +Elementals 
        [Splash]: "+1" damage to all enemies after strike once +Elementals 
        [Regeneration]: After taking damage, restore 1 HP +Druids 
        [Thorns]: Reflect 1 damage to attackers +Druids 
        [Inspire]: "+1/+1" until the end of battle +Knights 
        [Rally]: [Inspire] all allies at the start of battle +Knights 
        [Vengeful]: After Death, deals 1 damage to killer +Knights 
        [Shield]: The next damage this unit takes is reduced to 0 +Guardians 
        [Barrier]: Absorb 1 damage +Guardians 
        [Chaotic]: After strike deal 1 damage to random enemy +Demons {cm:2023-05-18T17:53:10}
        [Stoneskin]: Gain "+1" HP for each alive ally +Demons 
        [Bursting]: After Death, inflict 1 damage to all enemies +Demons 
    @heroes
        [Fury]
            +Orcs Berserker [4/4]: Start of battle: gain [Fury]
            +Orcs Fury Emissary [2/2]: [Fury] now gives 1 more ATK
            +Orcs +Druids Fury Warden [2/2]: When an ally gains [Fury], they also gain [Regeneration]
        [Enrage]
            +Orcs Blood Knight [4/2]: Turn start: Gain [Enrage]
            +Orcs Enrage Master [3/3]: [Enrage] now gives 1 more ATK
            +Orcs +Knights Frenzy Seer [2/2]: When an ally gains [Enrage], it also gives [Inspire] to adjacent units
        [Thorns]
            +Druids Thorns Enchanter [1/4]: [Thorns] now reflect 2 damage
            +Druids Thorns Mystic [1/4]: [Thorns] now also heal the unit for the damage reflected
            +Druids +Witches Thorns Artificer [1/4]: [Thorns] now give the attacker [Weakness]
            +Druids +Guardians Thorned Knight [2/4]: When an ally gains [Thorns], they also gain [Shield]
            +Druids +Warriors Thorned Spirit [1/4]: When an ally gains Thorns, they also gain [Might]
        [Regeneration]
            +Druids Regeneration Amplifier [2/2]: [Regeneration] now heals 2 HP
            +Druids Essence Keeper [2/2]: When an ally gains [Regeneration], it also applies to adjacent units
        [Inspire]
            +Knights Inspiring Herald [2/2]: [Inspire] now gives "+2/+2"
            +Knights Rune Forger [2/2]: After strike, grants [Inspire] to a random ally
            +Knights Goblin Alchemist [1/2]: At the end of turn, [Inspire] random ally
            +Knights +Guardians Barrier Shaper [3/3]: When an ally gains [Barrier], they also gain [Inspire]
        [Might]
            +Warriors +Knights  Valor Caller [2/2]: When an ally uses [Inspire], give all allies [Might]
            +Warriors +Guardians  Shield Savant [3/3]: [Shield] units now also gain [Might] when they block damage
            +Warriors +Guardians Shield Philosopher [3/3]: [Shield] units now gain [Might] every time they block damage
            +Warriors +Guardians Barrier Alchemist [3/3]: [Barrier] now gives the hero [Might] (2) when it blocks damage
            +Warriors +Guardians Guardian Knight [2/5]: When an ally gains [Shield], they also gain [Might]
            +Warriors Shadow Assassin [1/1]: Before strike, gain [Might] (2)
            +Warriors Vengeful Spirit [2/2]: After Death, gives all allies [Might]
            +Warriors Graveyard Warden [2/2]: When an ally dies, gain [Might]
            +Warriors Squire [1/2]: At the start of battle, grant [Might] to a random ally
        [Shield]
            +Guardians Celestial Sorcerer [1/4]: Turn end: give [Shield] to random ally
            +Guardians +Druids Holy Healer [2/4]: When an ally gains [Regeneration], they also gain [Shield]
            +Guardians +Medics Holy Guardian [3/4]: When an ally gains [Shield], they also gain [Blessing]
            +Guardians Shield Ascendant [3/3]: [Shield] units now reflect damage back to attacker
            +Guardians +Medics Blessing Enchanter [2/2]: [Blessing] now gives the hero [Shield] instead of "+1/+1"
        [Mend]
            +Medics Lifebinder [1/4]: [Mend] random injured ally at the start of each turn
        [Vitality]
            +Medics Gnome Mechanic [2/2]: Gives a random ally [Vitality] at the start of each turn
        [Blessing]
            +Medics Blessing Conduit [2/2]: When an gains [Blessing], it also applied to a random ally
            +Medics +Warriors Holy Avenger [3/3]: When an ally gains [Might], they also gain [Blessing]
            +Medics Blessing Amplifier [2/2]: [Blessing] gives "+1/+1" more
        [Honored]
            +Warriors Warchief [5/5]: Start of battle: gain [Honored]
        [Mark]
            +Witches Spectral Wraith [2/2]: After Death, apply [Mark] to killer
            +Witches Hexweaver [2/2]: When an enemy gains [Weakness], they also receive [Marked]
            +Witches Marked Prophet [2/1]: [Marked] now triples taken damage
            +Witches +Elementals Cursed Elemental [2/4]: [Splash] now also applies [Marked] to all enemies
        [Weakness]
            +Witches Weakness Seer [2/2]: [Weakness] "+1" ATK decrease
            +Witches Weakness Oracle [2/2]: [Weakness] now makes the enemy lose its next attack
            +Witches Succubus [2/3]: Before strike: apply [Weakness] to attacker
            +Demons +Witches Chaos Manipulator [2/2]: When enemy takes damage from [Chaotic], apply [Weakness]
        [Chaotic]
            +Demons Chaos Shaper [2/2]: [Chaotic] "+1" damage
            +Demons +Medics Soulstealer [3/2]: After an ally dies, gain [Chaotic] and [Vitality]
            +Demons +Medics Soul Harvester [3/3]: After an ally dies, gain [Chaotic] and [Blessing]
        [Splash]
            +Elementals +Witches Splash Magus [2/2]: [Splash] now gives all enemies [Weakness] instead of dealing damage
            +Elementals Splash Reaver [2/2]: [Splash] now deals "+1" damage
            +Elementals +Witches Infernal Mage [2/3]: [Splash] deals 1 extra damage for each [Weakness] on target
            +Elementals +Knights Radiant Paladin [3/4]: When an ally gains [Inspire], they also gain [Splash]
        +Elementals Stormcaller [2/2]: At the start of battle, deals 1 damage to all enemies
        +Elementals Arcane Illusionist [1/1]: At the start of battle, creates a copy of itself
#         Chaotic Illusionist [2/2]: Chaotic now swaps the HP and ATK of a random enemy instead of dealing damage
#         Guardian Golem [0/5]: Absorbs damage directed at adjacent heroes
#         Voidwalker [3/1]: Immune to the first attack each turn
#         Necromancer [1/3]: Summons a 1/1 Skeleton when an enemy hero dies
#         Mystic Sage [1/4]: At Battle end, if this hero survived, gain "+1/+1"
#         Gnome Tinkerer [2/3]: At the start of battle, summons a 1/1 Robot
    @enemies {c}
        Blighted Ghoul [2/2]: After Death, inflicts 1 damage to all enemies
        Spectral Apparition [1/1]: After Death, reduces the ATK of its killer by 1
        Eroding Golem [3/1]: After Death, deals 2 damage to killer
        Vengeful Wraith [2/2]: After Death, gives all other Vengeful Wraiths "+1/+1"
        Cursed Pharaoh [2/3]: After Death, curses all enemies, reducing their HP by "1"
        Sacrificial Cultist [1/1]: After Death, all other Sacrificial Cultists gain "+1/+1"
        Venomous Serpent [1/2]: After Death, poisons its killer, dealing "1" damage each turn
        Plague Rat [1/1]: After Death, infects all enemies with a disease, dealing "1" damage each turn
        Shadow Revenant [3/1]: After Death, reduces the HP of all enemies by "1"
        Doomsayer [1/1]: After Death, reduces the ATK of all enemies by "1"
        Spore Carrier [1/2]: After Death, spawns 2 "1/1" Sporelings


