function DBG(arg)
    print("Debug:", vim.inspect(arg))
end

function DBG_I(arg)
    vim.fn.input("Debug:", vim.inspect(arg))
end
