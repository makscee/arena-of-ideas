use super::*;

impl<T: 'static + Clone + Send + Sync> Table<T> {
    pub fn add_arena_leaderboard_columns(self, f: fn(&T) -> &TArenaLeaderboard) -> Self {
        self.column_int_dyn("flr", Box::new(move |d| f(d).floor as i32))
            .column_player_click_dyn("owner", Box::new(move |d| f(d).owner))
            .column_team_dyn("team", Box::new(move |d| f(d).team))
            .column_ts_dyn("time", Box::new(move |d| f(d).ts))
            .column_cstr_dyn("mode", Box::new(move |d, _| f(d).mode.cstr_expanded()))
    }
    pub fn add_content_vote_columns(self, f: fn(&T) -> String) -> Self {
        self.column_int_dyn("score", Box::new(move |d| todo!()))
            .column_btn_mod_dyn(
                "+",
                Box::new(move |d, _, _| todo!()),
                Box::new(move |d, _, b| b.active(todo!("get vote"))),
            )
            .column_btn_mod_dyn(
                "-",
                Box::new(move |d, _, _| todo!()),
                Box::new(move |d, ui, b| b.active(todo!("get vote")).red(ui)),
            )
    }
    pub fn add_content_favorite_columns(self, f: fn(&T) -> (String, String)) -> Self {
        self.column_int_dyn("fav", Box::new(move |d| todo!()))
            .column_btn_mod_dyn(
                "♥",
                Box::new(move |d, _, _| todo!()),
                Box::new(move |d, _, b| todo!()),
            )
    }
}
