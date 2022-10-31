use std::collections::VecDeque;

use super::*;

#[derive(clap::Args)]
pub struct CustomGame {
    #[clap(long)]
    config: std::path::PathBuf,
}

#[derive(geng::Assets, Deserialize)]
#[asset(json)]
struct CustomConfig {
    player: Vec<UnitType>,
    clans: HashMap<Clan, usize>,
    round: GameRound,
    fov: f32,
}

impl CustomGame {
    pub fn run(
        self,
        geng: &Geng,
        assets: &Rc<Assets>,
        shop_config: ShopConfig,
    ) -> Box<dyn geng::State> {
        let custom = futures::executor::block_on(<CustomConfig as geng::LoadAsset>::load(
            geng,
            &static_path().join(&self.config),
        ))
        .unwrap();
        let config = Config {
            player: custom.player,
            shop_player: vec![],
            clans: custom.clans,
            enemy_clans: hashmap! {},
            fov: custom.fov,
        };
        Box::new(Game::new(geng, assets, config, custom.round, true))
    }
}
