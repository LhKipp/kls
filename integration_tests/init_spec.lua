local log = require 'log'
local async = require 'plenary.async.tests'
local tfs = require 'tfs'
local golden = require 'golden_test'

async.describe("Test init", function()
    before_each(function()
        vim.cmd.cd(vim.env.KLS_PROJECT_DIR)
        vim.lsp.stop_client(vim.lsp.get_clients())
    end)

    async.it("Receives capabilities", function()
        local client = require "kserver".start("receives_capabilities")
        assert(client.server_capabilities ~= nil)
    end)

    async.it("Sets up project and source set scope nodes", function()
        local client = require "kserver".start("project_scope_node",
            { files = { ["src/main/kotlin/example.kt"] = "package example.com" } })
        local scopes = client.print_scopes({ print_file_contents = true })

        local t = GoldenTest:new("integration_tests/init_spec_golden.toml", "project_source_set_nodes")
        t:is_expected(scopes)
    end)
end)
