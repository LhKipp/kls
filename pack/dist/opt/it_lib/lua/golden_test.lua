--- Test file system
local log = require 'log'
local assert = require 'luassert'
local util = require 'util'

---@class GoldenTest
---@field test_name string
---@field file string
GoldenTest = {}

---@param test_name string
---@param file string
---@return GoldenTest
function GoldenTest:new(file, test_name)
    local test = { test_name = test_name, file = vim.env.KLS_PROJECT_DIR .. "/" .. file }
    setmetatable(test, self)
    self.__index = self
    return test
end

---@param actual string
function GoldenTest:is_expected(actual)
    if vim.env.GOLDEN_TEST_UPDATE == "1" then
        log.debug("Updating golden test", self.file)
        local new_file = util.system_stdout(
            { "toml", "set", self.file, 'tests.' .. self.test_name .. '.expected', actual }
        )
        local test_file = io.open(self.file, "w")
        assert(test_file ~= nil)
        test_file:write(new_file)
        test_file:close()
        return
    end

    local expected = util.system_stdout({
        "toml",
        "get",
        "--raw",
        self.file,
        'tests.' .. self.test_name .. '.expected',
    })
    assert.equal(vim.trim(expected), vim.trim(actual))
end
