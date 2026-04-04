#!/usr/bin/env bash
# check_wav.sh — verifies all .wav files in a directory are 32768 Hz, unsigned 8-bit PCM

[ "${CI}" = "true" ] && exit 0

TARGET_RATE=32768
TARGET_FORMAT="u8"   # ffprobe reports unsigned 8-bit as "u8"

DIR="${1:-.}"

if [[ ! -d "$DIR" ]]; then
    echo "Error: '$DIR' is not a directory" >&2
    exit 1
fi

shopt -s nullglob
wavs=("$DIR"/*.wav)

if [[ ${#wavs[@]} -eq 0 ]]; then
    echo "No .wav files found in '$DIR'"
    exit 0
fi

pass=0
fail=0

for f in "${wavs[@]}"; do
    name="$(basename "$f")"

    output=$(ffprobe -v error \
                -select_streams a:0 \
                -show_entries stream=sample_rate,sample_fmt \
                -of default=noprint_wrappers=1:nokey=1 \
                "$f" 2>/dev/null)

    fmt=$(echo "$output" | sed -n '1p')
    rate=$(echo "$output" | sed -n '2p')

    errors=()

    if [[ "$rate" != "$TARGET_RATE" ]]; then
        errors+=("sample rate is ${rate:-unknown} Hz (expected ${TARGET_RATE} Hz)")
    fi

    if [[ "$fmt" != "$TARGET_FORMAT" ]]; then
        errors+=("sample format is ${fmt:-unknown} (expected ${TARGET_FORMAT})")
    fi

    if [[ ${#errors[@]} -eq 0 ]]; then
        echo "  OK  $name"
        (( pass++ ))
    else
        echo " BAD  $name — $(IFS=', '; echo "${errors[*]}")"
        (( fail++ ))
    fi
done

echo ""
echo "Results: $pass passed, $fail failed (${#wavs[@]} files checked)"
[[ $fail -eq 0 ]]   # exit 0 if all pass, 1 otherwise