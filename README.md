Arena of Ideas is an innovative auto battler that aims to be created by its community as much as possible
# Goal
General aim is to create a game that is developed by collective intelligence of players that are invested in the game and really want to make & keep it fun. It shouldn't require any special skills, like coding. For a player it should be enough to have a general idea for what a hero should do. General inspiration and direction will be enough to contribute.

*The game has to become a machine that absorbs ideas, filters out best and integrates them*

With time, game will constantly evolve and thus never stagnate. Everything that is not fun is experienced by players and can be changed by them. If there is something you don't like, you could come back in a month and see it changed. 

# Competition
Global competition is on of the design pillars of the game. It should manifest in the gameplay as well as in content creation. We should always put spotlight on the best players and best idea contributors. This will depend a lot on the gameplay as well as on instruments for content creation.

# Gameplay
Currently, gameplay is a minimalistic auto battler, where core loop has 2 phases:
- **Shop** – you improve and manage your team of units
- **Battle** – you watch your team fight other team
Every unit in the game has 2 main stats: *HP* and *ATK*. Units can attack simultaneously each other, decreasing own *HP* value by *ATK* value of the attacker. When *HP* reaches zero, unit dies. If 2 units have same stats, on hit they will simply kill each other.

In battle, team units that are in front hit each other until on of the teams dies completely.

## Units
Game units are split in two categories: **Heroes** and **Enemies**. Player build a team out of heroes, and fights other player team or a team of enemies.

Unit is described by following things
### State
A state is simply a set of variables. Every unit has *HP* and *ATK* variables, for example. They also might have a *Name*, *Description*, or some values that are necessary to implement specific mechanics for their ability.

### House
Every unit belongs to at least one House.
House is a thematic container for game mechanics. It has a:
- name
- color
- abilities *(optional)*
- statuses *(optional)*

### Ability
Ability is a stored effect.

*e.g.* one of the simplest abilities is in the house of **Wizards** which is **Magic Missile**. It deals 1 damage to a random unit.

For a unit to use this ability, it has to be attached to a *Trigger*

### Trigger
Trigger is something that can use an ability (or custom effect) when some event happens.

*e.g.* on **Battle Start**: use **Magic Missile**

### Status
Status is a trigger that can be attached to any unit. It has a *state* which mainly contains *Charges* variable, that allows status stacking.

*e.g.* **Paladins** house has a **Shield** status, that negates next damage taken

Statuses can also modify state of a unit when active

*e.g.* **Holy** house has a **Blessing** status, that increases *ATK* and *HP* by 1 per *Charge*

### Innate Ability
Every unit can have an innate ability, which is a trigger that is a part of units' initial state. It is like a status for the most part, except it doesn't have charges and cannot be removed. This ability is what is described in units' description.

## Game Modes
### Global Tower
TBD

# Creation Instruments
TBD
