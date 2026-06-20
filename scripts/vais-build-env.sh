#!/usr/bin/env bash
# Shared build helper for internal compiler gates.
#
# Source this after HERE is set to the Vais repo root.

vais_build() {
    local src="${1:?vais_build needs a source path}"
    shift

    local trust_root trusted
    trust_root="$(mktemp -d)"
    trusted="$trust_root/compiler/self/$(basename "$src")"
    mkdir -p "$(dirname "$trusted")"
    cp "$src" "$trusted"
    VAISC_SELF_HOST_TRUST_ROOTS="${VAISC_SELF_HOST_TRUST_ROOTS:+$VAISC_SELF_HOST_TRUST_ROOTS:}$trust_root" \
        "$HERE/scripts/vaisc" build "$trusted" "$@"
}
