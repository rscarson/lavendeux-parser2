#!/bin/bash
# This script is used to fuzz the target program with the input file
# Usage: ./fuzzer.sh <optional phone number for notifications>

# If we are in the scripts directory, move to the parent directory
if [ -f "fuzzer.sh" ]; then
    cd ..
fi

cd fuzz
cargo update
rustup update
rustup override set nightly

if [ -n "$1" ]; then
    curl -X POST http://my_textbelt_server/text \
        -d number="$1" \
        -d "message=Fuzzer has started - notifications will be sent to this number."
fi

# Now we run the fuzzer in a loop
while true; do
    # Run the fuzzer
    cargo fuzz run parse

    # Now there should be a new file in the artifacts/parse directory
    # Let's get the contents of the newest file in the directory
    newest_file=$(ls -t artifacts/parse | head -n 1)
    newest_file_path="artifacts/parse/$newest_file"
    newest_file_contents=$(cat $newest_file_path)

    # Print it out in the form 'New fuzzer result in <filename>: <contents>'
    echo "\n\nNew fuzzer result in $newest_file_path\n-----\n$newest_file_contents\n"

    # If the user has provided a phone number, we will send a text message to that number
    # when the fuzzer finds a new result
    if [ -n "$1" ]; then
        curl -X POST http://my_textbelt_server/text \
            -d number="$1" \
            -d "message=New fuzzer result in '$newest_file_path':\n$newest_file_contents"
    fi
done