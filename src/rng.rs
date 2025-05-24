use rand::SeedableRng;
use rand::rngs::StdRng;

/// 获取 RNG, 当启用 benchmark 时由一个固定种子生成
pub fn get_rng() -> StdRng {
    if cfg!(feature = "benchmark") {
        StdRng::seed_from_u64(171)
    } else {
        StdRng::from_rng(&mut rand::rng())
    }
}
