-- Lua script that prints a summary of matches
if #results > 0 then
    local file_counts = {}
    for _, result in ipairs(results) do
        local file = result.file
        if not file_counts[file] then
            file_counts[file] = 0
        end
        file_counts[file] = file_counts[file] + 1
    end

    for file, count in pairs(file_counts) do
        print("File: " .. file .. " - Matches: " .. count)
    end
    print("Total matches found: " .. #results)
else
    print("No matches found.")
end
