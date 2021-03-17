#!/bin/sh

set -e
set -u

# Helper script to identify the bytecode files produced by `rustc -C save-temps`
#
#   --main: Get path of the actual main crate's bytecode file
#   -- alloc: Get path of the helper bytecode file containing Rust's alloc symbols

if [ -z ${LLVM_DIR+x} ]; then 
  LLVM_NM='llvm-nm'
else
  LLVM_NM="${LLVM_DIR}/bin/llvm-nm"
fi

if [ $# -lt 2 ] || { [ "$1" != '--main' ] && [ "$1" != '--alloc' ]; }; then
  echo 'Usage: get-bc.sh --main|--alloc <bc-files...>'
  exit 64
fi

if [ "$1" = '--main' ]; then
  needle='main'
fi
if [ "$1" = '--alloc' ]; then
  needle='__rust_alloc'
fi

shift

for file in "$@"; do
  if echo "$file" | grep -q '.no-opt.bc$'; then
    continue
  fi

  if $LLVM_NM --defined-only --just-symbol-name "${file}" | grep -q "^${needle}$"; then
    echo "${file}"
    exit 0
  fi
done

exit 1
