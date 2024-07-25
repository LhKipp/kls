local log = require 'log'
local tfs = require 'tfs'

local M = {}

--- @class KlsClient : vim.lsp.Client
--- @field print_scopes fun(): any

---Start KLS
---@param test_case_name string: opts for starting the server
---@param client_config vim.lsp.ClientConfig|nil: opts for starting the server
---@return KlsClient: The started client
M.start = function(test_case_name, client_config)
    vim.lsp.log.set_level("trace")

    local test_dir = tfs.test_dir({ files = { ["src/main/kotlin/example.kt"] = "package example.com" } })

    local client_config_fixed = vim.tbl_deep_extend("keep", client_config or {}, {
        name = "KLS Client - " .. test_case_name,
        cmd = {
            vim.fn.getcwd() .. "/target/debug/kls",
            "--log-file=logs/" .. test_case_name,
            "--start-new-log-file",
            "--log-timestamps=false",
        },
        cmd_env = { RUST_LOG = "trace" },
        root_dir = test_dir.root_dir,
        workspace_folders = test_dir.workspace_folders,
        -- handlers = {
        --     ["custom/printScopes"] = printScopesHandler
        -- }
    })
    local client_id = vim.lsp.start(client_config_fixed)
    assert(client_id ~= nil, "vim.lsp.start did not return a client_id")
    local client = vim.lsp.get_client_by_id(client_id)
    assert(client ~= nil, "vim.lsp.start did not return an existing client")
    vim.wait(5000, function() return client.initialized end, 100)
    assert(client.initialized == true)
    client.print_scopes = function(params)
        local kls_response, err = client.request_sync(
            "custom/printScopes",
            vim.tbl_deep_extend('keep', params or {}, {
                print_file_contents = false,
                trim_from_file_paths = test_dir.root_dir
            }),
            nil,
            0)
        assert(kls_response ~= nil, "Request failed")
        if err ~= nil then
            assert(err == "", "Err is not empty: " .. err)
        end
        log.info("return result", kls_response.result)
        return kls_response.result
    end
    return client
end

return M
