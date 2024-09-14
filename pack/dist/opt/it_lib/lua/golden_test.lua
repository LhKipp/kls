--- Test file system
local log = require 'log'
local assert = require 'luassert'
local util = require 'util'

---@class GoldenTestSpec
---@field file string
GoldenTestSpec = {}

---@class GoldenTest
---@field spec GoldenTestSpec
---@field test_name string
GoldenTest = {}

function default_golden_file()
    local str = debug.getinfo(2, "S").source:sub(2)
    local file_name_no_lua = str:gsub(vim.env.KLS_PROJECT_DIR .. "/", ""):gsub(".lua", "")
    assert(file_name_no_lua ~= nil, "Current file does not match default lua file regex")
    return file_name_no_lua .. "_golden.toml"
end

---@param file string
---@return GoldenTestSpec
function GoldenTestSpec:new(file)
    local spec = { file = vim.env.KLS_PROJECT_DIR .. "/" .. file }
    setmetatable(spec, self)
    self.__index = self
    return spec
end

function GoldenTest:new(spec, test_name)
    local test = { spec = spec, test_name = test_name }
    setmetatable(test, self)
    self.__index = self
    return test
end

---@param test_name string
---@return GoldenTest
function GoldenTestSpec:test(test_name)
    return GoldenTest:new(self, test_name)
end

---@param actual string
function GoldenTest:is_expected(actual)
    if vim.env.GOLDEN_TEST_UPDATE == "1" then
        log.debug("Updating golden test", self.spec.file)
        local new_file = util.system_stdout(
            { "toml", "set", self.spec.file, 'tests.' .. self.test_name .. '.expected', actual }
        )
        local test_file = io.open(self.spec.file, "w")
        assert(test_file ~= nil)
        test_file:write(new_file)
        test_file:close()
        return
    end

    local expected = util.system_stdout({
        "toml",
        "get",
        "--raw",
        self.spec.file,
        'tests.' .. self.test_name .. '.expected',
    })
    assert.equal(vim.trim(expected), vim.trim(actual))
end
