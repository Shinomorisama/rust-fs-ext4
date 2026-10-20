#!/usr/bin/env bash
set -euo pipefail
export RUSTUP_HOME=/workspace/.rustup CARGO_HOME=/workspace/.cargo
export PATH=/workspace/.cargo/bin:/workspace/onboarding/bin:$PATH
export CARGO_BUILD_JOBS=4
mkdir -p /workspace/onboarding/bin
if ! command -v rustup >/dev/null; then
  curl -fsSL https://sh.rustup.rs -o /tmp/onboarding-rustup.sh
  sh /tmp/onboarding-rustup.sh -y --no-modify-path --default-toolchain 1.95.0 --profile minimal --component rustfmt,clippy
fi
rustup toolchain install 1.95.0 --profile minimal --component rustfmt,clippy
chore_dir=$(mktemp -d /tmp/onboarding-chore.XXXXXX)
trap 'rm -rf "$chore_dir"' EXIT
base=https://github.com/antimatter-studios/chore/releases/download/v0.11.0
curl -fsSL "$base/chore-0.11.0-linux-x86_64.tar.gz" -o "$chore_dir/chore-0.11.0-linux-x86_64.tar.gz"
curl -fsSL "$base/checksums.txt" -o "$chore_dir/checksums.txt"
(cd "$chore_dir"; rg ' chore-0.11.0-linux-x86_64.tar.gz$' checksums.txt | sha256sum -c -; tar -xzf chore-0.11.0-linux-x86_64.tar.gz)
find "$chore_dir" -type f -name chore -exec install -m755 '{}' /workspace/onboarding/bin/chore \;
# Prepare the branch-specific core source view without changing sibling checkouts.
setup_root=/workspace/.cloud-setup
mkdir -p "$setup_root"
core_sha=9e900154a7dff2a6c8f136db31def33bed9f09cc
core_archive="$chore_dir/core-0.3.0.tar.gz"
curl -fsSL "https://codeload.github.com/antimatter-studios/rust-fs-core/tar.gz/$core_sha" -o "$core_archive"
printf '%s  %s\n' 87b91f7490d69b7f71c0975f50227583634b80fefbc20d88d9b8ffd875444af9 "$core_archive" | sha256sum -c -
mkdir "$chore_dir/core"
tar -xzf "$core_archive" --strip-components=1 -C "$chore_dir/core"
if test -e "$setup_root/pinned-core-0.3.0"; then
  diff -qr "$chore_dir/core" "$setup_root/pinned-core-0.3.0"
else
  mv "$chore_dir/core" "$setup_root/pinned-core-0.3.0"
fi
# proot supplies a process-local source view; no mount privilege or sibling rewrite.
# A fresh library-only checkout may have no sibling mount target yet.
mkdir -p /workspace/rust-fs-core
test "$(uname -m)" = x86_64
apt_root="$setup_root/apt"
mkdir -p "$apt_root/etc/parts" "$apt_root/etc/preferences.d" "$apt_root/lists/partial" "$apt_root/cache/archives/partial" "$setup_root/tools"
cat > "$apt_root/config" <<'APTCONFIG'
Dir::Etc "/workspace/.cloud-setup/apt/etc";
Dir::Etc::main "-";
Dir::Etc::parts "parts";
Dir::Etc::sourceparts "-";
Dir::State::lists "/workspace/.cloud-setup/apt/lists";
Dir::Cache "/workspace/.cloud-setup/apt/cache";
APTCONFIG
cat > "$apt_root/etc/sources.list" <<'APTSOURCES'
deb [signed-by=/usr/share/keyrings/debian-archive-keyring.gpg] https://deb.debian.org/debian trixie main
APTSOURCES
APT_CONFIG="$apt_root/config" /usr/bin/apt-get update
mkdir "$chore_dir/debs"
(cd "$chore_dir/debs"; APT_CONFIG="$apt_root/config" /usr/bin/apt-get download proot libtalloc2)
for package_file in "$chore_dir"/debs/*.deb; do
  dpkg-deb --extract "$package_file" "$setup_root/tools"
done
cat > "$setup_root/env.sh" <<'CASEFOLDENV'
export CARGO_HOME=/workspace/.cargo
export RUSTUP_HOME=/workspace/.rustup
export PATH=/workspace/onboarding/bin:/workspace/.cargo/bin:/usr/bin:$PATH
export CARGO_BUILD_JOBS=4
export CARGO_TARGET_DIR=/workspace/rust-fs-ext4/target/casefold-baseline
export XDG_CACHE_HOME=/workspace/.cache
export XDG_STATE_HOME=/workspace/.local/state
export FS_CORE_ROOT=/workspace/.cloud-setup/pinned-core-0.3.0
export RUSTFLAGS='-C linker-features=-lld'
CASEFOLDENV
cat > "$setup_root/run-casefold.sh" <<'CASEFOLDRUN'
#!/usr/bin/env bash
set -euo pipefail
source /workspace/.cloud-setup/env.sh
export LD_LIBRARY_PATH=/workspace/.cloud-setup/tools/usr/lib/x86_64-linux-gnu${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}
exec /workspace/.cloud-setup/tools/usr/bin/proot -b /workspace/.cloud-setup/pinned-core-0.3.0:/workspace/rust-fs-core "$@"
CASEFOLDRUN
chmod 755 "$setup_root/run-casefold.sh"
command -v gh >/dev/null
cd /workspace/rust-fs-ext4
core_version=$(python3 - <<'COREVERSION'
import tomllib
with open('Cargo.toml','rb') as f:
    print(tomllib.load(f)['dependencies']['rust-fs-core']['version'])
COREVERSION
)
case "$core_version" in
  0.3.0)
    "$setup_root/run-casefold.sh" cargo fetch --locked
    "$setup_root/run-casefold.sh" cargo build --locked --features cli
    "$setup_root/run-casefold.sh" chore test:unit
    ;;
  0.3.6)
    cargo fetch --locked
    cargo build --locked --features cli
    chore test:unit
    ;;
  *) printf 'Unreviewed core version: %s; follow branch AGENTS.md before building.\n' "$core_version" >&2; exit 1 ;;
esac
# Never change branches, sibling sources, dependency pins or lockfiles during setup.
# ext4-win-driver still needs a separately qualified Windows workflow.
