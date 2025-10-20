ct feat:
  cargo check {{ if feat == "none" { "" } else { "--features " + feat } }}

cbr feat:
  cargo build --release {{ if feat == "none" { "" } else { "--features " + feat } }}

crr feat nx ny ns:
  cargo run --release {{ if feat == "none" { "" } else { "--features " + feat } }} -- --nx {{nx}} --ny {{ny}} --ns {{ns}}

bench warmup run:
  just build benchmark
  hyperfine --warmup {{warmup}} -r {{run}} 'target/release/ray-tracing --nx 1200 --ny 800 --ns 100 --dry'
