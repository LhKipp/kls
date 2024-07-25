--- Test file system
local log = require 'log'
local Path = require 'plenary.path'

local M = {}

M.write = function(path, text)
    local p = Path:new(path)
    log.debug("Writing file at ", p.filename)
    p:parent():mkdir({ parents = true })

    local f, e = io.open(p.filename, "w")
    assert(f ~= nil, e)
    f:write(text)
    f:close()
end

M.test_dir = function(args)
    local dir = vim.system({ "mktemp", "-d" }, { text = true })
        -- Remove trailing newline via sub
        :wait().stdout:sub(1, -2) .. "/"

    log.debug("Setting up test in ", dir)

    vim.iter(args.files):map(function(k, v) return dir .. k, v end):each(M.write)

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

    M.write(dir .. "kls-test-project.json", vim.fn.json_encode(project))

    return project
end

return M
