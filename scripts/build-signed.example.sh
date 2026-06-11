#!/usr/bin/env bash
# 签名+公证构建模板：复制为 scripts/build-signed.sh（已 gitignore）并填入自己的凭据。
# 产出 aarch64 + x86_64 两个 DMG，均完成 .app 公证（tauri 自动）与 DMG 公证+装订（本脚本）。
set -euo pipefail
cd "$(dirname "$0")/.."
export PATH="$HOME/.cargo/bin:$PATH"

# 签名身份（钥匙串中的 Developer ID Application 证书，security find-identity -v -p codesigning 可查）
export APPLE_SIGNING_IDENTITY="Developer ID Application: <Your Name> (<TEAM_ID>)"

# 公证凭据：Apple ID + App 专用密码（https://account.apple.com/account/manage 生成）
export APPLE_ID="<your-apple-id@example.com>"
export APPLE_PASSWORD="<app-specific-password>"
export APPLE_TEAM_ID="<TEAM_ID>"

TARGETS=("aarch64-apple-darwin" "x86_64-apple-darwin")

for target in "${TARGETS[@]}"; do
  echo "==== 构建 $target ===="
  rustup target add "$target" >/dev/null 2>&1 || true
  pnpm tauri build --target "$target"

  dmg=$(ls "src-tauri/target/$target/release/bundle/dmg"/*.dmg | head -1)
  echo "==== DMG 公证: $dmg ===="
  xcrun notarytool submit "$dmg" \
    --apple-id "$APPLE_ID" --password "$APPLE_PASSWORD" --team-id "$APPLE_TEAM_ID" \
    --wait
  xcrun stapler staple "$dmg"
done

echo "==== 产物 ===="
for target in "${TARGETS[@]}"; do
  ls -lh "src-tauri/target/$target/release/bundle/dmg"/*.dmg
done
