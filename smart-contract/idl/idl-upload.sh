# Define variables
PROGRAM_ID="6HW8dXjtiTGkD4jzXs7igdFmZExPpmwUrRN5195xGup"
IDL_PATH="./codama-idl.json"
AUTHORITY="CbxqZdi1EQneomjjkCkZBmsQenHxEEfs5nDiZxveYoGB"
CLOSE_BUFFER=$(solana address)

# Create buffer and capture output
BUFFER_OUTPUT=$(npx @solana-program/program-metadata create-buffer "$IDL_PATH")
BUFFER=$(echo "$BUFFER_OUTPUT" | grep "buffer:" | cut -d' ' -f4)


# # Set buffer authority
npx @solana-program/program-metadata set-buffer-authority "$BUFFER" --new-authority "$AUTHORITY"

# Verify metadata program
# solana-verify verify-from-repo -um \
#   --program-id ProgM6JCCvbYkfKqJYHePx4xxSUSqJp7rh8Lyv7nk7S \
#   https://github.com/solana-program/program-metadata \
#   --commit-hash a41788e3ede0a388eb71c5122700cc63f19045cb \
#   --library-name spl_program_metadata

# # Write IDL
npx @solana-program/program-metadata write idl "$PROGRAM_ID" \
 --buffer "$BUFFER" \
 --export "$AUTHORITY" \
 --export-encoding base58 \
 --close-buffer "$CLOSE_BUFFER"