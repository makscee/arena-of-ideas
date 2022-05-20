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
    spawn_points: HashMap<SpawnPoint, Vec2<Coord>>,
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
            clans: custom.clans,
            spawn_points: custom.spawn_points,
            fov: custom.fov,
        };
        let shop = Shop::new(geng, assets, shop_config);
        Box::new(Game::new(geng, assets, config, shop, custom.round))
    }
}
