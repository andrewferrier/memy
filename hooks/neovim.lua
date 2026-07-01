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

local function cd_to_path(path)
    if path == nil or path == "" then
        return
    end

    vim.cmd({ cmd = "cd", args = { path } })
end

local file_list_command = "memy list -f | tac"
local dir_list_command = "memy list -d | tac"
local external_command_files = { "sh", "-c", file_list_command }
local external_command_dirs = { "sh", "-c", dir_list_command }

local fzf_ok, _ = pcall(require, "fzf-lua")
if fzf_ok then
    vim.api.nvim_create_user_command("MemyFZFFiles", function(_)
        require("fzf-lua").fzf_exec(file_list_command, {
            actions = require("fzf-lua.config").globals.actions.files,
            fzf_opts = {
                ["--exact"] = "",
                ["--no-sort"] = "",
            },
            previewer = "builtin",
        })
    end, {})

    vim.api.nvim_create_user_command("MemyFZFDirs", function(_)
        require("fzf-lua").fzf_exec(dir_list_command, {
            actions = {
                ["default"] = function(selected, _)
                    if selected and selected[1] then
                        cd_to_path(selected[1])
                    end
                end,
            },
            fzf_opts = {
                ["--exact"] = "",
                ["--no-sort"] = "",
            },
            previewer = "builtin",
        })
    end, {})

    vim.api.nvim_create_user_command("MemyFZF", function(_)
        vim.cmd.MemyFZFFiles()
    end, {})
end

local minipick_ok, _ = pcall(require, "mini.pick")
if minipick_ok then
    vim.api.nvim_create_user_command("MemyMiniPickFiles", function(_)
        require("mini.pick").builtin.cli({ command = external_command_files }, {})
    end, {})

    vim.api.nvim_create_user_command("MemyMiniPickDirs", function(_)
        local output = vim.system(external_command_dirs, { text = true }):wait()
        if output.code ~= 0 then
            vim.notify(
                "From external command: " .. output.stderr,
                vim.log.levels.ERROR
            )
            return
        end

        local items = {}
        for line in output.stdout:gmatch("[^\r\n]+") do
            table.insert(items, line)
        end

        if #items == 0 then
            return
        end

        require("mini.pick").start({
            source = {
                name = "Memy Dirs",
                items = items,
                choose = function(item)
                    cd_to_path(item)
                end,
            },
        })
    end, {})

    vim.api.nvim_create_user_command("MemyMiniPick", function(_)
        vim.cmd.MemyMiniPickFiles()
    end, {})
end

local ok_telescope, _ = pcall(require, "telescope.builtin")
if ok_telescope then
    local telescope_actions = require("telescope.actions")
    local telescope_action_state = require("telescope.actions.state")

    vim.api.nvim_create_user_command("MemyTelescopeFiles", function(_)
        require("telescope.builtin").find_files({
            find_command = external_command_files,
            sorter = require("telescope.sorters").fuzzy_with_index_bias(),
        })
    end, {})

    vim.api.nvim_create_user_command("MemyTelescopeDirs", function(_)
        require("telescope.builtin").find_files({
            find_command = external_command_dirs,
            sorter = require("telescope.sorters").fuzzy_with_index_bias(),
            attach_mappings = function(prompt_bufnr, _)
                telescope_actions.select_default:replace(function()
                    local entry = telescope_action_state.get_selected_entry()
                    telescope_actions.close(prompt_bufnr)
                    if entry and entry.value then
                        cd_to_path(entry.value)
                    end
                end)
                return true
            end,
        })
    end, {})

    vim.api.nvim_create_user_command("MemyTelescope", function(_)
        vim.cmd.MemyTelescopeFiles()
    end, {})
end
