local log = require 'log'
local async = require 'plenary.async.tests'
local tfs = require 'tfs'
local golden = require 'golden_test'

local function exec_keys(keys)
    local input = vim.api.nvim_replace_termcodes(keys, true, false, true)
    vim.api.nvim_feedkeys(input, 'x', false)
end

async.describe("DidChangeNotification", function()
    async.it("track code additions", function()
        local client = require "kserver".start("did_change__add_new_text",
            {
                files = {
                    ["src/main/kotlin/example.kt"] = "package example.com",
                }
            }
        )
        vim.cmd.edit("src/main/kotlin/example.kt")
        exec_keys("Go// hello world")

        -- assert exec_keys worked
        assert.equal(vim.api.nvim_buf_get_lines(0, -2, -1, false)[1], '// hello world')

        local scopes = client.print_scopes({ print_file_contents = true })
        assert.equal("", scopes)
        local t = GoldenTest:new("integration_tests/edit_buffer_spec_golden.toml", "did_change__add_new_text")
    end)
end)
