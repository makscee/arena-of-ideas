use super::*;

pub type NodeAsset = (String, u64, i32);

pub trait NodeAssetExt {
    fn new(data: String, owner_id: u64, rating: i32) -> Self;
    fn data(&self) -> &String;
    fn owner_id(&self) -> u64;
    fn rating(&self) -> i32;
    fn set_rating(&mut self, rating: i32);
}

impl NodeAssetExt for NodeAsset {
    fn new(data: String, owner_id: u64, rating: i32) -> Self {
        (data, owner_id, rating)
    }

    fn data(&self) -> &String {
        &self.0
    }

    fn owner_id(&self) -> u64 {
        self.1
    }

    fn rating(&self) -> i32 {
        self.2
    }

    fn set_rating(&mut self, rating: i32) {
        self.2 = rating;
    }
}

pub type LinkAsset = (u64, u64, String, String, i32, u8);

pub trait LinkAssetExt {
    fn new(
        parent_id: u64,
        child_id: u64,
        parent_kind: String,
        child_kind: String,
        rating: i32,
        solid: bool,
    ) -> Self;
    fn parent_id(&self) -> u64;
    fn child_id(&self) -> u64;
    fn parent_kind(&self) -> &String;
    fn child_kind(&self) -> &String;
    fn rating(&self) -> i32;
    fn set_rating(&mut self, rating: i32);
}

impl LinkAssetExt for LinkAsset {
    fn new(
        parent_id: u64,
        child_id: u64,
        parent_kind: String,
        child_kind: String,
        rating: i32,
        solid: bool,
    ) -> Self {
        (
            parent_id,
            child_id,
            parent_kind,
            child_kind,
            rating,
            solid as u8,
        )
    }

    fn parent_id(&self) -> u64 {
        self.0
    }

    fn child_id(&self) -> u64 {
        self.1
    }

    fn parent_kind(&self) -> &String {
        &self.2
    }

    fn child_kind(&self) -> &String {
        &self.3
    }

    fn rating(&self) -> i32 {
        self.4
    }

    fn set_rating(&mut self, rating: i32) {
        self.4 = rating;
    }
}
