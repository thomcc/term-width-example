#!/usr/bin/env bash
cargo run -- 'abc' 'lööps' 'ＡＢＣＤ' 'ᄀ' '각' 'ᄀᄀᄀ각ᆨᆨ' '👨‍👩‍👦‍👦' '🏳️‍🌈' '🇦🇶' 'Z̮̞̠͙͔ͅḀ̗̞͈̻̗Ḷ͙͎̯̹̞͓G̻O̭̗̮' '﷽'

# cargo run -- 'Z̮̞̠͙͔ͅḀ̗̞͈̻̗Ḷ͙͎̯̹̞͓G̻O̭̗̮' "$(printf "\e[32;1m%s\e[m" "this one is probably cheating")"
