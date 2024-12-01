test:
    rm -rf output
    mkdir output
    echo '# tmpl:test_chunk\nhello1' > output/file1.txt
    echo '# tmpl:test_chunk\nhello2' > output/file2.txt
    cargo install --path .
    templative test_template --output output --day_number 5
