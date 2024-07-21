local log = require 'log'

local M = {}

---Start KLS
---@param test_case_name string: opts for starting the server
---@param client_config vim.lsp.ClientConfig|nil: opts for starting the server
---@return vim.lsp.Client: The started client
M.start = function(test_case_name, client_config)
    vim.lsp.log.set_level("trace")
    local client_config_fixed = vim.tbl_deep_extend("keep", client_config or {}, {
        name = "KLS Client - " .. test_case_name,
        cmd = {
            vim.fn.getcwd() .. "/target/debug/kls",
            "--log-file=logs/" .. test_case_name,
            "--start-new-log-file",
            "--log-timestamps=false",
        },
        cmd_env = { RUST_LOG = "trace" }
    })
    local client_id = vim.lsp.start(client_config_fixed)
    assert(client_id ~= nil, "vim.lsp.start did not return a client_id")
    local client = vim.lsp.get_client_by_id(client_id)
    assert(client ~= nil, "vim.lsp.start did not return an existing client")
    vim.wait(5000, function() return client.initialized end, 100)
    assert(client.initialized == true)
    return client
end

return M
