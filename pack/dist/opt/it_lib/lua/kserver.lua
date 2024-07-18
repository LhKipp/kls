local M = {}

-- local log = require 'log'
--
M.hi = function() print("HI") end
M.start = function()
    -- local log = require'log'
    -- DBG(log)
    -- log.info("starting kls")
    vim.lsp.start { name = "kls", cmd = { "target/debug/kls" } }
end

return M
