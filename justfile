test:
    rm -rf output
    mkdir output
    echo '# tmpl:test_chunk\n------------------------\n# tmpl:test_chunk :append' > output/file1.txt
    echo '# tmpl:test_chunk\n------------------------\n# tmpl:test_chunk :append' > output/file2.txt
    cargo install --path .
    templative test_template --output output
    grep "2" output/file1.txt && echo "Test passed" || echo "\033[0;31m\nTest failed\n\033[0m"
    templative test_template --output output --day_number 5
    grep "5" output/file1.txt && echo "Test passed" || echo "\033[0;31m\nTest failed\n\033[0m"
