#!/bin/bash
set -e

mkdir -p ../toolchain/benchmarks/default/sources
mkdir -p ../toolchain/programs/afra
mkdir -p ../toolchain/workspace
git clone https://github.com/gem5/gem5.git ../toolchain/programs/gem5
cp assets/scripts/gem5_arm_script.py ../toolchain/programs/gem5_arm_script.py
mv src ../toolchain/programs/afra
mv Cargo.toml ../toolchain/programs/afra
mv README.md ../toolchain/programs/afra
rm -rf ../afra
cd ../toolchain/programs/gem5
scons build/ARM/gem5.opt -j $(nproc)
