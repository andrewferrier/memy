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
        require("fzf-lua").fzf_exec("memy list -f | tac", {
            actions = require("fzf-lua.config").globals.actions.files,
            fzf_opts = {
                ["--exact"] = "",
                ["--no-sort"] = "",
            },
            previewer = "builtin",
        })
    end, {})
end

local external_command = { "sh", "-c", "memy list -f | tac" }

local minipick_ok, _ = pcall(require, "mini.pick")
if minipick_ok then
    vim.api.nvim_create_user_command("MemyMiniPick", function(_)
        require("mini.pick").builtin.cli({ command = external_command }, {})
    end, {})
end

local ok_telescope, _ = pcall(require, "telescope.builtin")
if ok_telescope then
    vim.api.nvim_create_user_command("MemyTelescope", function(_)
        require("telescope.builtin").find_files({
            find_command = external_command,
            sorter = require("telescope.sorters").fuzzy_with_index_bias(),
        })
    end, {})
end
