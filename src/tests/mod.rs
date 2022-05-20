use crate::*;

mod clan;

use clan::*;

pub fn run_tests(assets: Assets) {
    test_clans(&assets);
}
