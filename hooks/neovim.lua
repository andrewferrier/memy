vim.api.nvim_create_autocmd({ "BufReadPre", "BufWritePost" }, {
    callback = function(event)
        local file = event.file

        if file:sub(1, 6) == "oil://" then
            file = file:sub(7)
        end

        vim.system({ "memy", "note", file }, { detach = true }, function(out)
            vim.schedule(function()
                if out.code ~= 0 then
                    vim.notify(
                        "From external command: " .. out.stderr,
                        vim.log.levels.ERROR
                    )
                end
            end)
        end)
    end,
})
