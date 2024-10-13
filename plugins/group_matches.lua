-- Lua script to group matches based on custom logic
local grouped_results = {}

-- Custom logic: group lines based on presence of specific keywords
for i = 1, #results do
    local result = results[i]
    local group_key

    if string.find(result.line_content, "ERROR") then
        group_key = "errors"
    elseif string.find(result.line_content, "WARN") then
        group_key = "warnings"
    else
        group_key = "others"
    end

    if not grouped_results[group_key] then
        grouped_results[group_key] = {}
    end

    table.insert(grouped_results[group_key], result)
end

-- Print grouped results
for group, matches in pairs(grouped_results) do
    print("Group: " .. group)
    for _, match in ipairs(matches) do
        print(string.format("  File: %s, Line: %d, Content: %s", match.file, match.line_number, match.line_content))
    end
end
