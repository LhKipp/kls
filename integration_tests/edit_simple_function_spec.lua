require 'golden_test'
local log = require 'log'
local async = require 'plenary.async.tests'
local util = require 'util'


local golden_spec = GoldenTestSpec:new(default_golden_file())

async.describe("DidChangeNotification updates the scope", function()
    local test_name = "did_change__add_new_simple_function"
    async.it(test_name, function()
        local client = require "kserver".start(test_name,
            {
                files = {
                    ["src/main/kotlin/example.kt"] = [[]],
                }
            }
        )
        vim.cmd.edit("src/main/kotlin/example.kt")
        util.exec_keys("gg^ifun myFun(myArg: Int): Int {\nreturn myArg\n}\n<ESC>")
        -- TODO, neovim is not sending the didChange notification without the write
        vim.cmd.write()

        local scopes = client.print_scopes({ print_file_contents = true, print_ast = true, print_scopes = true })
        golden_spec:test(test_name):is_expected(scopes)
    end)

    test_name = "did_change__update_existing_function"
    async.it(test_name, function()
        local client = require "kserver".start(test_name,
            {
                files = {
                    ["src/main/kotlin/example.kt"] = [[
fun myFun(myArg: Int) : Int {
    return myArg
}
                    ]],
                }
            }
        )
        vim.cmd.edit("src/main/kotlin/example.kt")
        util.exec_keys("/myFun<CR>ct{custom_fun(arg: Long): Long<ESC>")
        -- TODO, neovim is not sending the didChange notification without the write
        vim.cmd.write()

        local scopes = client.print_scopes({ print_file_contents = true, print_ast = true, print_scopes = true })
        golden_spec:test(test_name):is_expected(scopes)
    end)
    --
    -- test_name = "did_change__remove_existing_package_header"
    -- async.it(test_name, function()
    --     local client = require "kserver".start(test_name,
    --         {
    --             files = {
    --                 ["src/main/kotlin/example.kt"] = [[]],
    --             }
    --         }
    --     )
    --     vim.cmd.edit("src/main/kotlin/example.kt")
    --     util.exec_keys("ggipackage mypackage.com<Esc>")
    --     -- TODO, neovim is not sending the didChange notification without the write
    --     vim.cmd.write()
    --     util.exec_keys("ggdd<Esc>")
    --     vim.cmd.write()
    --
    --     log.info("sending print scopes request")
    --     local scopes = client.print_scopes({ print_file_contents = true, print_ast = true, print_scopes = true })
    --     golden_spec:test(test_name):is_expected(scopes)
    -- end)
end)