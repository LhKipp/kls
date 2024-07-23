local log = require 'log'

local M = {}

M.system_stdout = function(cmd)
    local result = vim.system(cmd):wait()
    assert.equal(result.stderr, '')
    assert.equal(result.signal, 0)
    return result.stdout
end

return M
