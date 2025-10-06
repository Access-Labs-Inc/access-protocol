# Define variables
PROGRAM_ID=
ANCHOR_IDL_PATH=
AUTHORITY=
CLOSE_BUFFER=

# Create buffer and capture output
BUFFER_OUTPUT=$(npx @solana-program/program-metadata create-buffer "$ANCHOR_IDL_PATH")
BUFFER=$(echo "$BUFFER_OUTPUT" | grep "buffer:" | cut -d' ' -f4)

# Set buffer authority
npx @solana-program/program-metadata set-buffer-authority "$BUFFER" --new-authority "$AUTHORITY"

# Write IDL
npx @solana-program/program-metadata write idl "$PROGRAM_ID" \
 --buffer "$BUFFER" \
 --export "$AUTHORITY" \
 --export-encoding base58 \
 --close-buffer "$CLOSE_BUFFER"