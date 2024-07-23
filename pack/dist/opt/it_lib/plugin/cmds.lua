local log = require 'log'

function DBG(arg)
    log.debug(vim.inspect(arg))
end

function DBG_I(arg)
    vim.fn.input("Debug:", vim.inspect(arg))
end
