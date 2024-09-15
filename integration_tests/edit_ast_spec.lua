require 'golden_test'
local log = require 'log'
local async = require 'plenary.async.tests'
local util = require 'util'


local golden_spec = GoldenTestSpec:new(default_golden_file())

async.describe("DidChangeNotification updates the ast", function()
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
        util.exec_keys("GOclass B()<ESC>")
        -- TODO, neovim is not sending the didChange notification without the write
        vim.cmd.write()

        local scopes = client.print_scopes({ print_file_contents = true, print_ast = true })
        golden_spec:test("did_change__add_new_text_ast"):is_expected(scopes)
    end)

    async.it("track code edits", function()
        local client = require "kserver".start("did_change__change_text_ast",
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
        util.exec_keys("ggWcwmypackage<esc>j^WcwB")
        -- TODO, neovim is not sending the didChange notification without the write
        vim.cmd.write()

        local scopes = client.print_scopes({ print_file_contents = true, print_ast = true })
        golden_spec:test("did_change__change_text_ast"):is_expected(scopes)
    end)

    async.it("track code edits", function()
        local client = require "kserver".start("did_change__delete_text_ast",
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
        util.exec_keys("ggWdwx")
        -- TODO, neovim is not sending the didChange notification without the write
        vim.cmd.write()

        local scopes = client.print_scopes({ print_file_contents = true, print_ast = true })
        golden_spec:test("did_change__delete_text_ast"):is_expected(scopes)
    end)
end)
