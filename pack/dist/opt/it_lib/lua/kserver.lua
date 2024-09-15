local log = require 'log'
local tfs = require 'tfs'

local M = {}

--- @class KlsClient : vim.lsp.Client
--- @field print_scopes fun(params: table): any

---Start KLS
---@param test_case_name string: opts for starting the server
---@param client_config vim.lsp.ClientConfig|nil: opts for starting the server
---@return KlsClient, table: The started client
M.start = function(test_case_name, test_dir_cfg, client_config)
    vim.lsp.log.set_level("trace")

    -- Create autocmd for debugging
    vim.api.nvim_create_autocmd('LspNotify', {
        callback = function(args)
            -- local bufnr = args.buf
            -- local client_id = args.data.client_id
            local method = args.data.method
            local params = args.data.params

            log.trace("Send notification ", method, params)
        end,
    })

    local test_dir = tfs.create(test_case_name, test_dir_cfg)

    -- setup nvim before starting the lsp, to have proper setup
    vim.cmd.cd(test_dir.root_dir)
    vim.cmd.edit(test_dir.root_dir .. "/kls-test-project.json")

    -- extend client_config with defaults
    local client_config_fixed = vim.tbl_deep_extend("keep", client_config or {}, {
        name = "KLS Client - " .. test_case_name,
        cmd = {
            vim.env.KLS_PROJECT_DIR .. "/target/debug/kls",
            "--log-file=logs/" .. test_case_name,
            "--start-new-log-file",
            "--log-timestamps=false",
        },
        root_dir = test_dir.root_dir,
        workspace_folders = test_dir.workspace_folders,
    })

    -- Add FileType autocmd, so that below started lsp receives didChange notifications
    vim.api.nvim_create_autocmd('FileType', {
        pattern = 'kotlin',
        callback = function()
            log.debug("FileType autocmd send")
            vim.lsp.start(client_config_fixed)
        end,
    })

    -- start lsp
    local client_id = vim.lsp.start(client_config_fixed)
    assert(client_id ~= nil, "vim.lsp.start did not return a client_id")
    local client = vim.lsp.get_client_by_id(client_id)
    assert(client ~= nil, "vim.lsp.start did not return an existing client")
    vim.wait(5000, function() return client.initialized end, 100)
    assert(client.initialized == true)

    -- setup custom lsp commands
    client.print_scopes = function(params)
        local kls_response, err = client.request_sync(
            "custom/printScopes",
            vim.tbl_deep_extend('keep', params or {}, {
                trim_from_file_paths = test_dir.root_dir
            }),
            nil,
            0)
        assert(kls_response ~= nil, "Request failed")
        if err ~= nil then
            assert(err == "", "Err is not empty: " .. err)
        end
        log.debug("print_scopes result: ", kls_response.result)
        return kls_response.result
    end


    return client, test_dir
end

return M
