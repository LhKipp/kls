--- Test file system
local log = require 'log'

local M = {}

M.test_dir = function(args)
    local dir = vim.system({ "mktemp", "-d" }, { text = true })
        -- Remove trailing newline via sub
        :wait().stdout:sub(1, -2) .. "/"
    log.debug("Setting up test in ", dir)
    local project = {
        id = 1,
        name = args.name or "KLS Test",
        root_dir = dir,
        workspace_folders = { { name = dir, uri = "file://" .. dir } },
        source_sets = args.source_sets or {
            {
                name = "kotlin",
                src_dir = "src/main/kotlin",
                dependencies = {}
            },
            {
                name = "test",
                src_dir = "src/main/test",
                dependencies = {
                    {
                        kind = "SourceSet",
                        name = "kotlin",
                        visibility = "Api"
                    }
                }
            }
        }
    }
    local defs_file = io.open(dir .. "kls-test-project.json", "w")
    assert.not_nil(defs_file)
    defs_file:write(vim.fn.json_encode(project))
    defs_file:close()
    return project
end

return M
