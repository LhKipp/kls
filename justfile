alias it := integration-test

# by default run all tests
export RUST_LOG := "trace" 
export KLS_TEST_LOG := "info"
export KLS_PROJECT_DIR := `pwd`

integration-test test="integration_tests" :
    nvim --headless --noplugin -u integration_tests/run_tests.vim -c \
        "PlenaryBustedDirectory {{test}} {minimal_init = 'integration_tests/testrc.vim'}"
