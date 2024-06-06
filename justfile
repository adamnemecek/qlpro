set shell := ["nu", "-l", "-c"]
[private]
default:
   @just --list

deploy:
   cargo build --release
   cp target/release/qlpro /Users/adamnemecek/.cargo/bin/
