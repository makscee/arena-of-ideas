Screen box anchoring @ui
Text shader constant scale, size depending on lines @ui
Move u_scale into box @shaders
Status get ability vars from caster instead of owner @gameplay
Show stats after status on card
Parallel hero ratings run with discrepancy test @content
API for data structs: clear()
Stats panel working in battle (?) @ui
Build
    Card rework @visual {cm:2023-06-20T20:24:25}
    Colors rework {start:2023-06-22T14:01:32} {cm:2023-06-22T19:17:15} {duration:05h15m}
    Statuses pool {start:2023-06-26T13:16:42} {c} {cm:2023-06-26T14:55:03} {duration:01h38m}
        pool take {cm:2023-06-26T14:51:31}
        rarity display {cm:2023-06-26T14:51:25}
    Shop reroll {start:2023-06-28T13:10:37} {cm:2023-06-28T13:34:04} {duration:23m}
    Gallery {start:2023-06-26T19:43:22} {cm:2023-06-27T17:10:03} {duration:21h26m}
    Push text @ui {cm:2023-07-07T16:46:19}
        panel working {start:2023-06-19T13:49:25} {cm:2023-06-19T14:47:17} {duration:57m}
        g change {cm:2023-06-21T15:25:50}
        gain team status {cm:2023-06-23T15:35:57}
    Stats info panel @ui {cm:2023-07-08T15:18:51}
        panel working {start:2023-06-19T14:47:43} {cm:2023-06-19T15:17:25} {duration:29m}
        g {cm:2023-06-19T15:55:08}
        total score {cm:2023-07-08T15:05:55}
        team status {cm:2023-06-23T17:07:51}
        level {cm:2023-06-19T15:55:11}
    Hover hint @ui {cm:2023-06-22T21:26:28} {c}
        panel working {start:2023-06-19T15:42:09} {cm:2023-06-19T15:55:22} {duration:13m}
        hover working {cm:2023-06-21T15:25:35}
        unit statuses {cm:2023-06-21T15:25:37}
        shop buy btns {cm:2023-06-22T21:26:24}
        start battle btn {cm:2023-06-22T21:26:24}
        definitions {cm:2023-06-21T15:25:41}
    Alert text @ui {cm:2023-07-07T16:45:44}
        battle end {start:2023-06-23T18:37:12} {cm:2023-06-23T19:09:45} {duration:32m}
    Alert cards @ui {start:2023-06-20T13:59:26} {cm:2023-06-23T16:45:35} {duration:3d_02h46m} {c}
        panel working {cm:2023-06-20T16:34:39}
        hero buy {cm:2023-06-20T16:34:41}
        enemy choose {start:2023-06-21T15:31:58} {cm:2023-06-21T16:20:26} {duration:48m}
        status choose {start:2023-06-22T12:52:30} {cm:2023-06-22T20:41:28} {duration:07h48m}
        team status choose {start:2023-06-23T14:59:39} {cm:2023-06-23T15:35:54} {duration:36m}
        aoe status {start:2023-06-23T15:45:37} {cm:2023-06-23T15:48:34} {duration:02m}
    Shop buttons @gameplay {cm:2023-06-23T16:45:24} {c}
        buy hero {cm:2023-06-20T21:02:46}
        buy status {cm:2023-06-22T13:25:48}
        buy aoe status {cm:2023-06-23T15:50:05}
        buy team status {cm:2023-06-23T15:36:23}
        buy slot {start:2023-06-23T15:50:07} {cm:2023-06-23T15:54:23} {duration:04m}
    Game over screen @ui {cm:2023-07-04T13:12:56} {c}
        restart {cm:2023-07-04T13:12:54}
        score {cm:2023-07-04T13:12:55}
    Core loop: spend g on statuses & heroes -> battle & get g -> sacrifice 1+ & get g -> restart @gameplay {start:2023-06-23T18:36:22} {cm:2023-06-26T15:27:58} {duration:2d_20h51m}
    No sacrifice for single hero team @gameplay {cm:2023-06-26T15:27:47}
    Chain all actions @visual {cm:2023-07-04T21:08:01}
    Max rank 3 @gameplay {cm:2023-06-26T15:27:50}
    Rank up every 10 rounds {cm:2023-07-02T13:32:52}
    Rework enemy generation {start:2023-07-01T15:15:46} {cm:2023-07-07T16:45:50} {duration:6d_01h30m}
    ~50 Total heroes @content {cm:2023-07-05T22:02:35}
    ~20 Total enemies @content {cm:2023-07-05T22:02:40}
    Limit slots buying {cm:2023-07-07T16:43:29}
    Simplified ladder format {start:2023-07-08T15:59:35} {cm:2023-07-08T18:06:38} {duration:02h07m}
    Unnified Shop offers {cm:2023-07-25T22:31:03}
    Ladder @content
        ladder generation from heroes {cm:2023-07-17T21:36:10}
        single team ladder {cm:2023-07-25T14:59:37}
        iterative ladder generation {cm:2023-07-25T14:59:38}
        auto-ladder play mode {cm:2023-08-01T17:18:59}
        alert "new enemy added" {cm:2023-08-01T17:19:03}
        separate initial from generated {cm:2023-08-02T19:02:04}
    Curses
        rank up enemies, no sacrifice
        buff enemies {cm:2023-07-26T20:21:26}
        get g {cm:2023-07-26T20:21:23}
        rank up one
        rank up enemies
    Rank up after battle {cm:2023-07-27T17:19:53}
    Occasional sacrifice {cm:2023-07-27T17:19:54}
    Reroll price {cm:2023-07-27T17:19:55}
    Tape navigation buttons
    Game rule hint panels
        ATK & HP
        Buffs
    Slot number indicator {cm:2023-07-31T17:03:08}
    UI/UX
        Buff apply focused
        Next enemy panel top-right
    Balance buffs {cm:2023-08-01T17:25:04}
    Sacrifice on demand {cm:2023-08-01T18:29:22}
    Main Menu {cm:2023-08-02T19:01:37}
        resume ladder {cm:2023-08-02T19:01:40}
        new ladder {cm:2023-08-02T19:01:41}
    Continuous ladder generation {cm:2023-08-02T19:01:27}
Socials
    Build on itch.io
    Build post on reddit
    Build post on tg
    Build devlog on yt
