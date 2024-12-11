#!/usr/bin/env bash

set -e

ROOT_DIR="$(git rev-parse --show-toplevel)"
KEYS_DIR="${ROOT_DIR}/light-prover/proving-keys"

mkdir -p "$KEYS_DIR"

INCLUSION_26_BUCKET="bafybeiacecbc3hnlmgifpe6v3h3r3ord7ifedjj6zvdv7nxgkab4npts54"
NON_INCLUSION_26_BUCKET="bafybeigp64bqx2k2ogwur4efzcxczm22jkxye57p5mnmvgzvlpb75b66m4"
NON_INCLUSION_40_BUCKET="bafybeigp64bqx2k2ogwur4efzcxczm22jkxye57p5mnmvgzvlpb75b66m4"
COMBINED_26_26_BUCKET="bafybeigp64bqx2k2ogwur4efzcxczm22jkxye57p5mnmvgzvlpb75b66m4"
COMBINED_26_40_BUCKET="bafybeigp64bqx2k2ogwur4efzcxczm22jkxye57p5mnmvgzvlpb75b66m4"
APPEND_WITH_PROOFS_BUCKET="bafybeicngrfui5cef2a4g67lxw3u42atyrfks35vx4hu6c4rme3knh6lby"
APPEND_WITH_SUBTREES_BUCKET="bafybeieyujtdrhp52unqkwvzn36o4hh4brsw52juaftceaki4gfypszbxa"
APPEND_ADDRESS_BUCKET="bafybeib2rajatndlpslpqhf4vrbekpyyehjt5byivfzxl36c5p67ypddvu"
UPDATE_BUCKET="bafybeievf2qdaex4cskdfk24uifq4244ne42w3dghwnnfp4ybsve6mw2pa"

get_bucket_url() {
    local FILE="$1"
    if [[ $FILE == inclusion_26_* ]]; then
        echo "https://${INCLUSION_26_BUCKET}.ipfs.w3s.link/${FILE}"
    elif [[ $FILE == non-inclusion_26_* ]]; then
        echo "https://${NON_INCLUSION_26_BUCKET}.ipfs.w3s.link/${FILE}"
    elif [[ $FILE == non-inclusion_40_* ]]; then
        echo "https://${NON_INCLUSION_40_BUCKET}.ipfs.w3s.link/${FILE}"
    elif [[ $FILE == combined_26_26_* ]]; then
        echo "https://${COMBINED_26_26_BUCKET}.ipfs.w3s.link/${FILE}"
    elif [[ $FILE == combined_26_40_* ]]; then
        echo "https://${COMBINED_26_40_BUCKET}.ipfs.w3s.link/${FILE}"
    elif [[ $FILE == append-with-proofs_* ]]; then
        echo "https://${APPEND_WITH_PROOFS_BUCKET}.ipfs.w3s.link/${FILE}"
    elif [[ $FILE == append-with-subtrees_* ]]; then
        echo "https://${APPEND_WITH_SUBTREES_BUCKET}.ipfs.w3s.link/${FILE}"
    elif [[ $FILE == address-append_* ]]; then
        echo "https://${APPEND_ADDRESS_BUCKET}.ipfs.w3s.link/${FILE}"
    elif [[ $FILE == update_* ]]; then
        echo "https://${UPDATE_BUCKET}.ipfs.w3s.link/${FILE}"
    fi
}

case "$1" in
    "light")
        SUFFIXES=(
            "inclusion_26:1 2 3 4 8"
            "non-inclusion_26:1 2 3 4 8"
            "non-inclusion_40:1 2 3 4 8"
            "combined_26_26:1_1 1_2 1_3 1_4 2_1 2_2 2_3 2_4 3_1 3_2 3_3 3_4 4_1 4_2 4_3 4_4"
            "combined_26_40:1_1 1_2 1_3 1_4 2_1 2_2 2_3 2_4 3_1 3_2 3_3 3_4 4_1 4_2 4_3 4_4"
            "append-with-proofs_26:1 10"
            "append-with-subtrees_26:1 10"
            "update_26:1 10"
            "address-append_40:1 10"
        )
        ;;
    "full")
        SUFFIXES=(
            "inclusion_26:1 2 3 4 8"
            "non-inclusion_26:1 2 3 4 8"
            "non-inclusion_40:1 2 3 4 8"
            "combined_26_26:1_1 1_2 1_3 1_4 2_1 2_2 2_3 2_4 3_1 3_2 3_3 3_4 4_1 4_2 4_3 4_4"
            "combined_26_40:1_1 1_2 1_3 1_4 2_1 2_2 2_3 2_4 3_1 3_2 3_3 3_4 4_1 4_2 4_3 4_4"
            "append-with-proofs_26:1 10 100 500 1000"
            "append-with-subtrees_26:1 10 100 500 1000"
            "update_26:1 10 100 500 1000"
            "address-append_40:1 10 100 250 500 1000"
        )
        ;;
    *)
        echo "Usage: $0 [light|full]"
        exit 1
        ;;
esac

for group in "${SUFFIXES[@]}"; do
    base=${group%:*}
    suffixes=${group#*:}
    for suffix in $suffixes; do
        for ext in key vkey; do
            file="${base}_${suffix}.${ext}"
            url="$(get_bucket_url "$file")"
            echo "Downloading $file"
            curl -S --retry 3 -o "${KEYS_DIR}/${file}" "$url"
        done
    done
done