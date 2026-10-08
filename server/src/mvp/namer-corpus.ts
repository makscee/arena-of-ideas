// Fusion names for measuring the crude check (crude.ts, fusions.test.ts).

// Innocent fantasy names that must all pass: common heads and tails glued
// (seeded draw), plus words the #594 merge checks found refused.
export const INNOCENT_NAMES: readonly string[] = [
  "Ashdust", "Ashforge", "Ashhorn", "Ashrat", "Bloodbrand", "Blooddrake", "Blooddust", "Bloodfang", "Bloodhide", "Bloodhound", "Bloodseer",
  "Bloodvein", "Bonebinder", "Bonecrest", "Bonedust", "Boneforge", "Boneling", "Bonereaper", "Bramblehorn", "Bramblehound", "Brambleling",
  "Bramblelord", "Briarbinder", "Briarborn", "Briardust", "Briarhound", "Briarroot", "Briarshard", "Briarspawn", "Briarspire", "Briarvein",
  "Briarwarden", "Brinebrand", "Brinedrake", "Brinefang", "Brinehide", "Brineking", "Brinemaw", "Brinerat", "Coalbinder", "Coalcrest",
  "Coalfang", "Coalking", "Coalseer", "Coalshard", "Coalthorn", "Dawndust", "Dawnforge", "Dawnheart", "Dawnhound", "Dawnlord", "Dawnsnare",
  "Dawnspawn", "Dreadbloom", "Dreadclaw", "Dreadhorn", "Dreadling", "Dreadlord", "Dreadmender", "Duskbinder", "Duskcrest", "Duskheart",
  "Duskhide", "Duskhorn", "Dusklord", "Duskmender", "Duskshade", "Emberbane", "Embercrest", "Emberhide", "Embermender", "Fangbane",
  "Fangblade", "Fangborn", "Fangdust", "Fanghorn", "Fanghound", "Fangshade", "Fangshard", "Fangthorn", "Flintbinder", "Flintheart",
  "Flinthorn", "Flintmaw", "Flintmender", "Flintroot", "Flintshade", "Frostblade", "Frostcaller", "Frostdust", "Frostmaw", "Frostrat",
  "Frostshade", "Frostsnare", "Galecrest", "Galedrake", "Galemender", "Galerat", "Galescar", "Galewraith", "Glasscrest", "Glassdust",
  "Glasshide", "Glassling", "Glasslord", "Glassward", "Glasswraith", "Gloombrand", "Gloomcrest", "Gloomdust", "Gloomseer", "Gloomsnare",
  "Gloomvein", "Gravebane", "Gravecrest", "Gravedust", "Gravehide", "Graveking", "Gravespawn", "Gravevein", "Gravewarden", "Grimfang",
  "Grimforge", "Grimscar", "Grimsnare", "Grimwing", "Hailgnaw", "Hailheart", "Hailhound", "Hailwarden", "Hailwraith", "Hexcaller",
  "Hexlord", "Hexmaw", "Hexseer", "Hexshade", "Hollowclaw", "Hollowdust", "Hollowmender", "Hollowseer", "Hollowshard", "Hollowvein",
  "Ironblade", "Ironhorn", "Ironhound", "Ironspire", "Manaborn", "Manabrand", "Manadust", "Manareaper", "Mindclaw", "Mindgnaw",
  "Mindspawn", "Mindspire", "Mindwarden", "Mistbrand", "Mistcaller", "Mistdrake", "Mistrat", "Mistseer", "Mooncrest", "Moonheart",
  "Moonlord", "Moonroot", "Moonshade", "Moonsnare", "Moonvein", "Moonwing", "Mossclaw", "Mossfang", "Mossheart", "Mosshound", "Mossling",
  "Mosslord", "Mossspire", "Nightbloom", "Nightbrand", "Nightheart", "Nightvein", "Nightwarden", "Plaguebane", "Plaguebloom",
  "Plagueforge", "Plaguehide", "Plagueking", "Plagueling", "Plagueshard", "Plaguevein", "Plagueward", "Roseblade", "Rosebloom",
  "Rosedrake", "Rosegnaw", "Rosemender", "Roseshade", "Rosesnare", "Rotblade", "Rotgnaw", "Rothorn", "Rotseer", "Rotsnare", "Rotspire",
  "Rotvein", "Rotweaver", "Runeborn", "Runecaller", "Runeclaw", "Runecrest", "Runehide", "Runethorn", "Runevein", "Scythebinder",
  "Scythehide", "Scythehorn", "Scytherat", "Scytheroot", "Scythespawn", "Shadowcrest", "Shadowhide", "Shadowhorn", "Shadowmender",
  "Shadowrat", "Shadowshard", "Shadowspire", "Soulbrand", "Soulforge", "Soulhide", "Soullord", "Soulrat", "Soulshard", "Soulsnare",
  "Soulward", "Soulwing", "Spiceling", "Spiceshard", "Spicesnare", "Spicespire", "Sporecaller", "Sporeking", "Sporelord", "Sporemender",
  "Sporereaper", "Sporesnare", "Sporeward", "Starshade", "Starshard", "Starward", "Starweaver", "Stingbinder", "Stinghide", "Stingscar",
  "Stingshade", "Stingspire", "Stingwarden", "Stingwing", "Stormbloom", "Stormborn", "Stormbrand", "Stormcaller", "Stormdrake",
  "Stormfang", "Stormheart", "Stormling", "Stormmender", "Stormroot", "Stormscar", "Stormshade", "Stormshard", "Sunblade", "Sundust",
  "Sunhound", "Thornheart", "Thornking", "Thornling", "Thornvein", "Thornwarden", "Tidebloom", "Tidedust", "Tidethorn", "Tideward",
  "Venomdrake", "Venomdust", "Venommaw", "Venomrat", "Venomspawn", "Voidbloom", "Voidrat", "Voidshard", "Wildforge", "Wildheart",
  "Wildhide", "Wildling", "Wildmender", "Wildscar", "Wildseer", "Wildwraith", "Wingbinder", "Wingforge", "Winggnaw", "Winghorn",
  "Stardust", "Nightshade", "Bonereaper", "Manalith", "Spicefang", "Cuirass", "Thornwing", "Cuckoo-clock", "Sauerkraut", "Pakistan",
  "Yiddish", "Japan", "Stardrake", "Embarrass", "Benign", "Moonignite", "Mustardseed", "Knightfall", "Jewelwing", "Cockatrice",
  "Butterfly", "Assassin", "Titanfang", "Glassblade", "Hourglass", "Thornyfang", "Bloodsucker", "Soulsnatcher", "Honeysuckle",
  "Scatterclaw", "Mosscumulus", "Farseer", "Starseed", "Cutlass", "Compass", "Buttress", "Peacock", "Raccoon", "Cocoon", "Spoonbill",
  "Homogenous", "Scrapper", "Analog",
  // the #609 merge check's collateral, and words that start or end like a new stem
  "Starseeker", "Starserpent", "Starsentinel", "Warserpent", "Warsentinel", "Enigma", "Twinkle", "Spicy", "Starseer", "Briarseer",
  "Warmonger", "Ironmonger", "Fearmonger", "Darklance", "Dusklance", "Blackland", "Ashthorn", "Ashtalon", "Marshtide", "Fleshtearer",
  "Brushtail", "Wishtide", "Ash Titan", "Fortitude", "Altitude", "Gratitude", "Entity", "Identity", "Petition", "Moonogre",
  "Dragon Ogre", "Egyptian", "Impervious", "Pimpernel", "Farthing", "Bump", "Bumper", "Harass", "Kvass",
  "Titmouse", "Grapevine", "Boobytrap", "Trapeze", "Gypsum", "Tardigrade", "Cumbersome", "About", "Above", "Abomination", "Bonebuster",
  "Bumblebee", "Album", "Knobble", "Stitchling", "Stormscrap", "Glassface", "Glasshead", "Grasshead", "Brasshead",
];

// Every fusion name in m1's MVP world on 2026-10-06 (mvp_fusions, read-only copy).
export const LIVE_NAMES: readonly string[] = [
  "Ash's Despoiler", "Aurora Seeder", "Batrak", "Benevolent Touched", "Blasted Servant", "Blessed Regenerator", "Blooming Crusade",
  "Blossom Berserker", "Brave Mentor", "Brook Troop", "Cavalry Mentor", "Charger's Healer", "Clawed Hunter", "Clawraptor",
  "Crimson Steward", "Doomsayer", "Fadest Sov'em", "Fang Fiend", "Flourish Scribe", "Flourish's Healer", "Frostcaller", "Grub Colossus",
  "Guarded Risen", "Guardstone", "Heist Digger", "Hungerbringer", "Inspire Bearer", "Ironcaller", "Leach Tinker", "Leatherneck",
  "Malific Coach", "Misery Uniter", "Misr Attack", "Mistwhisperer", "Moonborne Saint", "Murderous Rat", "Plague Rat's Spawn",
  "Plagueleader", "Rampage Surge", "Roseguard", "Screech Rat", "Scythe of Menace", "Scythe Root", "Seed Shifter", "Shield Knight",
  "Shroud Wanderer", "Silent Surge", "Soul Satiated", "Soulcaller Icebind", "Sowing Saboteur", "Squito Giver", "Stromwerk Master",
  "Syndrome Surgeon", "Synerge Syphonator", "Thorn Bearer", "Tidebreaker", "Toxic Wanderer", "Twilight Shot", "Twilight Weaver",
  "Venomous Berserker", "Venomous Champion", "Victim Reaper", "Warbringer Kataplex", "Web Overseer", "Whispering Desolator", "Whisperwing",
  "Wispspinner", "Wycherly",
];

// Every name Qwen3-4B gave in the R2-4 bench (docs/round2/namer/sum-qwen3-4b-s-habit.txt).
// The raw out-*.jsonl were never committed; these are all the bench names kept.
export const BENCH_NAMES: readonly string[] = [
  "Aegisbond", "Aetherblight", "Aetherkeep", "Ashnibb", "Balancerr", "Balancert", "Balancier", "Balorock", "Blastrotor", "Bleedtender",
  "Blightflare", "Blightreap", "Bloomreel", "Bloomwarden", "Brawlaxe", "Briarpawn", "Cinderward", "Circuitheal", "Circuitmend",
  "Clipmnder", "Coralgloom", "Crownflare", "Crownreap", "Decreeborn", "Decreeborne", "Decryser", "Dewtender", "Dodgekeep", "Dreadcell",
  "Dreadlash", "Dreadsiphon", "Duskflare", "Earmolder", "Eelwhisper", "Eldrivale", "Fainthold", "Faltalean", "Fangbind", "Fangflare",
  "Feyreaper", "Firesovereign", "Flamestead", "Flamevein", "Flamewarden", "Fleecenave", "Flicksnoop", "Flowerpierce", "Flowerpunch",
  "Ghastlyron", "Glareshade", "Gloomspire", "Glowthorn", "Gnatoverse", "Gripplague", "Gritclash", "Gritsnip", "Grubbone", "Hamagun",
  "Heal", "Healion", "Healogen", "Healstorm", "Healtheon", "Healward", "Hearthingeather", "Holyflame", "Honeburst", "Hushblade",
  "Hushbreaker", "Hygienecurse", "Joltbasher", "Jolttherapist", "Joltveil", "Joltwell", "Lhemlock", "Lichvolt", "Mercinder", "Mightflare",
  "Missporch", "Mortspade", "Munkberry", "Necroheal", "Nextrix", "Nighbattler", "Nighflare", "Noctscourge", "Noctscurry", "Nocture",
  "Omenflesh", "Pactshriek", "Pesttangle", "Petaldart", "Petaldrain", "Petaldrip", "Pulsehealer", "Punisherclaw", "Radiantheal",
  "Rageleveler", "Ravenousk Ing", "Rootwarden", "Roseredirect", "Ruinscar", "Ruinwatcher", "Rustcloak", "Rustskitter", "Rustwarden",
  "Sackmire", "Sanguimend", "Sanguincense", "Sanguiscope", "Sanguiscribe", "Sanguisent", "Sanguisire", "Scurgeflame", "Sewermender",
  "Sewersage", "Sewerscourge", "Sewgrowler", "Shadekeep", "Shadewarden", "Shadowskeet", "Shardsnare", "Shieldblight", "Shieldbolt",
  "Shieldmend", "Shieldspire", "Shieldtender", "Shockbite", "Shockrotter", "Shotblight", "Sickbloom", "Sicknessspire", "Sickreap",
  "Sicktwine", "Sickwardent", "Siphonmend", "Siphroot", "Siphthorn", "Skelethar", "Snarehand", "Snatchwisp", "Snipmind", "Snipwatch",
  "Soulflare", "Soulhealer", "Spearloom", "Spikebloom", "Spikeflame", "Spikeinfest", "Sporpunch", "Steadturn", "Steelhand", "Steelward",
  "Stillblight", "Stillblot", "Stillsmite", "Stingerock", "Stitchpunch", "Stoutclad", "Stoutclash", "Stowturn", "Stunrock", "Stunshell",
  "Sundering", "Sundermute", "Sunderveil", "Swellfeed", "Swordtend", "Syringaider", "Syringus", "Tackspike", "Tanglefeed", "Tearspike",
  "Thornsore", "Throneveil", "Thronewhisper", "Thronewitch", "Tidebreaker", "Tincturebite", "Trenchspit", "Vanguard", "Vaporsquire",
  "Vermireg", "Vexroot", "Vineguard", "Vinehandler", "Wardrop", "Wardspike", "Wardwane", "Wastekeeper", "Wipeguard", "Wispstancher",
  "Woundweaver", "Zappest", "Zapshadow",
];

// Every crude entry of the hand lists before #594's LDNOOBW round (fusions.ts at
// 2146a195: crude BLOCKED_STEMS and BLOCKED_WORDS, CRUDE, EDGE_CRUDE, "spic"):
// each must still be refused as a name on its own (fusions.test.ts).
export const CRUDE_BEFORE_594: readonly string[] = [
  "abo", "anal", "anus", "arse", "arsehole", "ass", "assclown", "asses", "assface", "asshat", "asshead", "asshole", "asskick", "asslick",
  "asswipe", "auschwitz", "badass", "ballsack", "bastard", "bastards", "beaner", "beaners", "bellend", "bestiality", "beyotch", "biatch",
  "bitch", "bitches", "blowjob", "bollock", "bollocks", "boner", "boob", "boobies", "boobs", "boong", "bukkake", "bullshit", "butt",
  "butthole", "buttplug", "cameljockey", "chinaman", "chingchong", "chink", "choad", "clit", "cock", "cocks", "cocksucker", "coolie",
  "coon", "cripple", "cuck", "cum", "cumbucket", "cumdump", "cumlord", "cumming", "cumrag", "cumshot", "cumslut", "cunt", "cunts", "dago",
  "darkie", "darky", "dick", "dickhead", "dickwad", "dildo", "dong", "dothead", "douche", "dumbass", "dyke", "erection", "fag", "faggot",
  "fags", "fap", "fatass", "fck", "felch", "fuck", "fucker", "fuckface", "fucking", "fuhrer", "fuk", "fuq", "fvck", "gangbang",
  "gaschamber", "gay", "gays", "gestapo", "golliwog", "gook", "greaser", "gringo", "gyp", "gypo", "gyppo", "gypsy", "halfbreed", "handjob",
  "heeb", "heil", "heilhitler", "hencummer", "hentai", "hitler", "holocaust", "homo", "honkey", "honky", "horny", "hymie", "incest",
  "injun", "jackass", "jap", "jerkoff", "jew", "jewkill", "jewkiller", "jigaboo", "jihadi", "jism", "jizz", "junglebunny", "kaffir",
  "kafir", "kickass", "kike", "killgays", "killjew", "kkk", "klan", "klansman", "knobhead", "kraut", "kyke", "ladyboy", "lesbo", "libtard",
  "lolicon", "lynch", "lyncher", "lynching", "masturbat", "masturbate", "masturbator", "midget", "milf", "minge", "mofo", "molest",
  "molester", "mong", "mongoloid", "motherfucker", "nazi", "nazis", "necrophil", "necrophile", "negro", "nig", "niga", "nigg", "nigga",
  "niggas", "nigger", "nigguh", "nog", "nonce", "nutsack", "orgasm", "orgy", "paedo", "paedophil", "paki", "pedo", "pedophil", "pedophile",
  "penis", "phuck", "phuk", "pikey", "piss", "pissed", "pisser", "poof", "poofter", "poontang", "porchmonkey", "porn", "porno", "pouf",
  "prick", "pricks", "pube", "pussies", "pussy", "queef", "raghead", "rape", "raper", "rapist", "redskin", "retard", "retarded", "rimjob",
  "saboner", "sambo", "sandmonkey", "sandnigger", "schlong", "scrotum", "semen", "sex", "sexmancer", "sexslave", "sextim", "sexy",
  "shemale", "shit", "shite", "shithead", "shitskin", "sht", "siegheil", "skank", "slanteye", "slut", "sluts", "smartass", "smegma",
  "smut", "sodom", "sodomite", "sodomy", "spastic", "spaz", "sperm", "spic", "spick", "spics", "spunk", "squaw", "swastika", "tard",
  "testic", "testicle", "thot", "tits", "titties", "tosser", "towelhead", "trannie", "trannies", "tranny", "turd", "twat", "twats",
  "vagina", "wank", "wanker", "wetback", "whitepower", "whitepride", "whitey", "whore", "whores", "wigger", "wog", "wogs", "wop", "yid",
  "yids", "yotch", "zipperhead",
];

// Names a mask let through when it reached past its innocent words (R2-17):
// each must be refused (fusions.test.ts).
export const MASK_LEAKS: readonly string[] = [
  "Ashthead", "Bigtitude", "Mongeroid", "Boneshter", "Embershtide", "Moontity", "Kingfarther", "Tithead", "Titlord", "Twinklord",
  "Aurashthorn", "Lunarshtide", "Auramongrat", "Auracoonrat", "Lunarklandrake", "Lunarnogreaper",
  // pass 2: a stem glued behind a head the mask's own letters let through
  "Ashthole", "Fishthead", "Marshthead", "Kingnogre", "Darknogre", "Blacknogre", "Gravenogre", "Raventity", "Krakentity", "Sirentity",
  "Wardentity", "Raventitle", "Imptitude", "Deeptitude", "Mystictitude", "Rattitude", "Coraltitude", "Lancertitude", "Bigtitle",
  "Kingtition", "Kingtwinkle", "Frostarserat", "Shadowarserat", "Crystalcumlord", "Mosstardlord", "Bonenigmaw", "Pyrosemendrake",
  "Fangrapevine", "Frostrapeze", "Stormanality", "Analogre", "Bonegyptail", "Kingspicy", "Crystalbum",
];

// Innocent names the tightened masks must still pass (R2-17): the collateral
// of the first tightening, and the words the anchored masks are for.
export const MASK_INNOCENT: readonly string[] = [
  "Ashtooth", "Fishtail", "Darkland", "Thornogre", "Sanctity", "Tithe", "Title", "Farther", "Racoon", "Warmongering", "Among Stars",
  // pass 2: one-word compounds of two whole roots, and the words the anchored masks are for
  "Flashtail", "Splashtail", "Thrashtail", "Lashtail", "Bashtail", "Clashthorn", "Flashthorn", "Crashtide", "Ashtide", "Ashtail",
  "Ashtimber", "Ashtiger", "Clashtalon", "Crashtalon", "Washtail", "Ravenogre", "Dragonogre", "Ironogre", "Nonentity", "Aptitude",
  "Ineptitude", "Rectitude", "Attitude", "Latitude", "Platitude", "Certitude", "Entitle", "Subtitle", "Competition", "Partition",
  "Superstition", "Bustard", "Analogy", "Talcum", "Twinkling",
];

// Toilet words and the short stems broad masks hid (R2-17), glued to heads
// that end in a vowel, r, n, p, c, g, k or w as well (fusions.test.ts): every
// one must be refused.
export const GLUED_STEMS: readonly string[] = [
  "fart", "poop", "crap", "piss", "turd", "bum", "pimp", "perv", "gimp", "knob", "erect", "hymen", "tit", "tits", "twink", "sht",
  "shit", "shite", "mong", "mongoloid", "coon", "klan", "nog",
];
