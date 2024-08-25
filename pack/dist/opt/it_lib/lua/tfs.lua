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

M.create = function(test_case_name, passed_args)
    local args = passed_args or {}

    local dir = "/tmp/" .. test_case_name
    log.debug("Setting up test in ", dir)
    vim.system({ "rm", "-r", dir }):wait()
    vim.system({ "mkdir", dir }):wait()
    vim.system({ "mkdir", "-p", dir .. "/src/main/kotlin" }):wait()
    vim.system({ "mkdir", "-p", dir .. "/src/main/test" }):wait()

    vim.iter(args.files or {}):map(function(k, v) return dir .. "/" .. k, v end):each(M.write)

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

    M.write(dir .. "/kls-test-project.json", vim.fn.json_encode(project))

    return project
end

return M
