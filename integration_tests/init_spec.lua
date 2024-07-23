local log = require 'log'
local async = require 'plenary.async.tests'
local tfs = require 'tfs'
local golden = require 'golden_test'

async.describe("Test init", function()
    async.it("Receives capabilities", function()
        local client = require "kserver".start("receives_capabilities")
        assert(client.server_capabilities ~= nil)
    end)

    async.it("Sets up project and source set scope nodes", function()
        local client = require "kserver".start("project_scope_node")
        local scopes = client.print_scopes()

        local t = GoldenTest:new("integration_tests/init_spec_golden.toml", "project_source_set_nodes")
        t:is_expected(scopes)
    end)
end)
