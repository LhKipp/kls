require 'golden_test'
local log = require 'log'
local async = require 'plenary.async.tests'
local util = require 'util'


local golden_spec = GoldenTestSpec:new(default_golden_file())

async.describe("DidChangeNotification update the ast", function()
    async.it("track code additions", function()
        local client = require "kserver".start("did_change__add_new_text_ast",
            {
                files = {
                    ["src/main/kotlin/example.kt"] = [[
package example.com
class A()
]],
                }
            }
        )
        vim.cmd.edit("src/main/kotlin/example.kt")
        util.exec_keys("GkOclass B()<ESC>")
        -- TODO, neovim is not sending the didChange notification without the write
        vim.cmd.write()

        local scopes = client.print_scopes({ print_file_contents = true, print_ast = true })
        golden_spec:test("did_change__add_new_text_ast"):is_expected(scopes)
    end)

    --     async.it("track code edits", function()
    --         local client = require "kserver".start("did_change__change_text",
    --             {
    --                 files = {
    --                     ["src/main/kotlin/example.kt"] = "package example.com",
    --                 }
    --             }
    --         )
    --         vim.cmd.edit("src/main/kotlin/example.kt")
    --         exec_keys("$Bcwmypackage<ESC>")
    --         -- TODO, neovim is not sending the didChange notification without the write
    --         vim.cmd.write()
    --
    --         -- assert exec_keys worked
    --         assert.equal(vim.api.nvim_buf_get_lines(0, 0, 1, false)[1], 'package mypackage.com')
    --
    --         local scopes = client.print_scopes({ print_file_contents = true })
    --         golden_spec:test("did_change__change_text"):is_expected(scopes)
    --     end)
    --
    --     async.it("track code edits", function()
    --         local client = require "kserver".start("did_change__delete_text",
    --             {
    --                 files = {
    --                     ["src/main/kotlin/example.kt"] = [[package example.com
    -- ]],
    --                 }
    --             }
    --         )
    --         vim.cmd.edit("src/main/kotlin/example.kt")
    --         exec_keys("dd")
    --         -- TODO, neovim is not sending the didChange notification without the write
    --         vim.cmd.write()
    --
    --         -- assert exec_keys worked
    --         -- assert.equal(vim.api.nvim_buf_get_lines(0, 0, 1, false)[1], '')
    --
    --         local scopes = client.print_scopes({ print_file_contents = true })
    --         golden_spec:test("did_change__delete_text"):is_expected(scopes)
    --     end)
end)
