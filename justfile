test:
    rm -rf output
    mkdir output
    echo '# test_chunk\nhello' > output/file1.txt
    cargo install --path .
    templative test_template --output output --day_number 5
