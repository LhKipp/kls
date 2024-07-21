local log = require 'log'
local async = require 'plenary.async.tests'

async.describe('Test init', function()
    async.it('Receives capabilities', function()
        local client = require 'kserver'.start(
            "receives_capabilities"
        )
        assert(client.server_capabilities ~= nil)
    end)
end)
