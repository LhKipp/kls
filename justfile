# Logs of the language server. Logs are written to /tmp/<test>/logs
export RUST_LOG := env_var_or_default("RUST_LOG","trace,parser=info") 
# Logs of the test setup
export KLS_TEST_LOG := env_var_or_default("KLS_TEST_LOG", "info")
# Pass 1 to update golden files
export GOLDEN_TEST_UPDATE := env_var_or_default("GOLDEN_TEST_UPDATE","0")
# The root directory of the project
export KLS_PROJECT_DIR := `pwd`

[doc("Run integration tests
`> just it [spec]`
By default all integration tests are run
")]
integration-test test="integration_tests" :
    nvim --headless --noplugin -u integration_tests/run_tests.vim -c \
        "PlenaryBustedDirectory {{test}} {minimal_init = 'integration_tests/testrc.vim'}"
alias it := integration-test
