#!/bin/bash
npm run tauri build

sudo install -m 755 src-tauri/target/release/i4archmenu /usr/local/bin/i4archmenu
cp src-tauri/target/release/i4archmenu ~/.config/IHyprVN/apps