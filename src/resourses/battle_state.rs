use super::*;

#[derive(Deserialize, TypeUuid, TypePath, Debug, Clone)]
#[uuid = "6cb61798-ec2c-4875-bec8-464c4f56c229"]
pub struct BattleState {
    pub left: PackedTeam,
    pub right: PackedTeam,
}
