#!/bin/sh

RUSTDOCFLAGS="-D warnings" cargo doc --document-private-items
