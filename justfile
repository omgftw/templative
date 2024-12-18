test:
    #! /bin/bash
    rm -rf output
    mkdir output
    echo -e '# tmpl:test_chunk\n------------------------\n# tmpl:test_chunk :append' > output/file1.txt
    echo -e '# tmpl:test_chunk\n------------------------\n# tmpl:test_chunk :append' > output/file2.txt
    cargo install --path .
    templative test_template --output output
    grep "2" output/file1.txt && echo "Test passed" || echo -e "\033[0;31mTest failed: Expected '2' in output/file1.txt\033[0m"
    templative test_template --output output --day_number 5
    grep "5" output/file1.txt && echo "Test passed" || echo -e "\033[0;31mTest failed: Expected '5' in output/file1.txt\033[0m"
    # Ensure all files exist
    ls output/file1.txt && echo "Test passed" || echo -e "\033[0;31mTest failed: output/file1.txt does not exist\033[0m"
    ls output/file2.txt && echo "Test passed" || echo -e "\033[0;31mTest failed: output/file2.txt does not exist\033[0m"
    ls output/test_rewrite/file1.txt && echo "Test passed" || echo -e "\033[0;31mTest failed: output/test_rewrite/file1.txt does not exist\033[0m"
    ls output/test_rewrite/file2.txt && echo "Test passed" || echo -e "\033[0;31mTest failed: output/test_rewrite/file2.txt does not exist\033[0m"
    ls output/test_rewrite/file5.txt && echo "Test passed" || echo -e "\033[0;31mTest failed: output/test_rewrite/file5.txt does not exist\033[0m"
    ls output/test_rewrite/filetest_rewrite.txt && echo "Test passed" || echo -e "\033[0;31mTest failed: output/test_rewrite/filetest_rewrite.txt does not exist\033[0m"
    ls output/subdir1/subdir-file1.txt && echo "Test passed" || echo -e "\033[0;31mTest failed: output/subdir1/subdir-file1.txt does not exist\033[0m"
    # Check contents of files
    diff output/file1.txt test-validations/file1.txt && echo "Test passed" || echo -e "\033[0;31mTest failed: Content mismatch in output/file1.txt vs test-validations/file1.txt\033[0m"
    diff output/file2.txt test-validations/file2.txt && echo "Test passed" || echo -e "\033[0;31mTest failed: Content mismatch in output/file2.txt vs test-validations/file2.txt\033[0m"
