it:
	KLS_PROJECT_DIR=${PWD} KLS_TEST_LOG=info RUST_LOG=trace \
		nvim --headless --noplugin -u integration_tests/run_tests.vim -c \
		"PlenaryBustedDirectory integration_tests/edit_buffer_spec.lua {minimal_init = 'integration_tests/testrc.vim'}"
