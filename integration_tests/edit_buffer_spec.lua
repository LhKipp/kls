require 'golden_test'
local async       = require 'plenary.async.tests'
local log         = require 'log'
local util        = require 'util'

local golden_spec = GoldenTestSpec:new(default_golden_file())

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
        util.exec_keys("Go// hello world<ESC>")
        -- TODO, neovim is not sending the didChange notification without the write
        vim.cmd.write()

        -- assert exec_keys worked
        assert.equal(vim.api.nvim_buf_get_lines(0, -2, -1, false)[1], '// hello world')

        local scopes = client.print_scopes({ print_file_contents = true })
        golden_spec:test("did_change__add_new_text"):is_expected(scopes)
    end)

    async.it("track code edits", function()
        local client = require "kserver".start("did_change__change_text",
            {
                files = {
                    ["src/main/kotlin/example.kt"] = "package example.com\n",
                }
            }
        )
        vim.cmd.edit("src/main/kotlin/example.kt")
        util.exec_keys("$Bcwmypackage<ESC>")
        -- TODO, neovim is not sending the didChange notification without the write
        vim.cmd.write()

        -- assert exec_keys worked
        assert.equal(vim.api.nvim_buf_get_lines(0, 0, 1, false)[1], 'package mypackage.com')

        local scopes = client.print_scopes({ print_file_contents = true })
        golden_spec:test("did_change__change_text"):is_expected(scopes)
    end)

    async.it("track code deletions", function()
        local client = require "kserver".start("did_change__delete_text",
            {
                files = {
                    ["src/main/kotlin/example.kt"] = [[package example.com
]],
                }
            }
        )
        vim.cmd.edit("src/main/kotlin/example.kt")
        util.exec_keys("ggdd")
        -- TODO, neovim is not sending the didChange notification without the write
        vim.cmd.write()

        -- assert exec_keys worked
        -- assert.equal(vim.api.nvim_buf_get_lines(0, 0, 1, false)[1], '')

        local scopes = client.print_scopes({ print_file_contents = true })
        golden_spec:test("did_change__delete_text"):is_expected(scopes)
    end)
end)
