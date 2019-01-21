PPROF_BINARY_PATH=$PWD/target/release/ pprof -http : -source_path=$PWD:"$(rustc --print sysroot)/lib/rustlib/src/rust/src" target/release/partition main.profile
