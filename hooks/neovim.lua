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

local fzf_ok, _ = pcall(require, "fzf-lua")
if fzf_ok then
    vim.api.nvim_create_user_command("MemyFZF", function(_)
        require("fzf-lua").fzf_exec("memy list -f", {
            actions = require("fzf-lua.config").globals.actions.files,
            fzf_opts = {
                ["--exact"] = "",
                ["--no-sort"] = "",
            },
            previewer = "builtin",
        })
    end, {})
end

local minipick_ok, _ = pcall(require, "mini.pick")
if minipick_ok then
    vim.api.nvim_create_user_command("MemyMiniPick", function(_)
        require("mini.pick").builtin.cli(
            { command = { "memy", "list", "-f" } },
            {}
        )
    end, {})
end
