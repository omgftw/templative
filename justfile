test:
    rm -rf output
    mkdir output
    echo '# tmpl:test_chunk\n------------------------\n# tmpl:test_chunk :append' > output/file1.txt
    echo '# tmpl:test_chunk\n------------------------\n# tmpl:test_chunk :append' > output/file2.txt
    cargo install --path .
    templative test_template --output output --day_number 5
