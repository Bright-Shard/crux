#!/bin/sh

cargo test --profile dev-unwind -Zbuild-std= -p crux-macros -p crux-macros-impl -p crux-rust-ast
