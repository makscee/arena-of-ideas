Screen box anchoring @ui
Text shader constant scale, size depending on lines @ui
Build
    Card rework @visual {cm:2023-06-20T20:24:25}
    Colors rework {start:2023-06-22T14:01:32} {cm:2023-06-22T19:17:15} {duration:05h15m}
    Statuses pool
    Show stats after status on card
    Push text @ui
        panel working {start:2023-06-19T13:49:25} {cm:2023-06-19T14:47:17} {duration:57m}
        g change {cm:2023-06-21T15:25:50}
        gain team status {cm:2023-06-23T15:35:57}
        var change
        enemy rank defeated
    Stats info panel @ui
        panel working {start:2023-06-19T14:47:43} {cm:2023-06-19T15:17:25} {duration:29m}
        g {cm:2023-06-19T15:55:08}
        total score
        team status {cm:2023-06-23T17:07:51}
        team vars
        level {cm:2023-06-19T15:55:11}
    Hover hint @ui {cm:2023-06-22T21:26:28} {c}
        panel working {start:2023-06-19T15:42:09} {cm:2023-06-19T15:55:22} {duration:13m}
        hover working {cm:2023-06-21T15:25:35}
        unit statuses {cm:2023-06-21T15:25:37}
        shop buy btns {cm:2023-06-22T21:26:24}
        start battle btn {cm:2023-06-22T21:26:24}
        definitions {cm:2023-06-21T15:25:41}
    Alert text @ui
        battle start
        battle end
        sacrifice start
        shop start
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
    Game over screen @ui
        restart
        score
    Core loop: spend g on statuses & heroes -> battle & get g -> sacrifice 1+ & get g -> restart @gameplay
    No sacrifice for single hero team @gameplay
    Chain all actions @visual
    Max rank 3 @gameplay
    ~50 Total heroes @content
    ~20 Total enemies @content
    15 (45) balanced levels @content
