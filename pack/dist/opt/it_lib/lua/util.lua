local log = require 'log'

local M = {}

M.system_stdout = function(cmd)
    local result = vim.system(cmd):wait()
    assert.equal(result.stderr, '')
    assert.equal(result.signal, 0)
    return result.stdout
end

M.exec_keys = function(keys)
    local input = vim.api.nvim_replace_termcodes(keys, true, false, true)
    vim.api.nvim_feedkeys(input, 'x', false)
end


return M
