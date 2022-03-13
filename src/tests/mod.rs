use crate::*;

mod alliance;

use alliance::*;

pub fn run_tests(assets: Assets) {
    test_alliances(&assets);
}
