-- Lua script to generate an advanced Markdown report of matches
local md_file = io.open("search_report.md", "w")

-- Write Markdown header and general information
md_file:write("# Search Matches Report\n\n")
md_file:write("_Automatically generated report summarizing the search results._\n\n")

-- Write metadata about the report generation
md_file:write("**Generated On:** " .. os.date("%Y-%m-%d %H:%M:%S") .. "\n")
md_file:write("**Total Matches Found:** " .. #results .. "\n\n")

-- Group results by files to make the report more organized
local grouped_results = {}
for i = 1, #results do
    local result = results[i]
    if not grouped_results[result.file] then
        grouped_results[result.file] = {}
    end
    table.insert(grouped_results[result.file], result)
end

-- Write a summary section
md_file:write("## Summary of Matches by File\n\n")
for file, matches in pairs(grouped_results) do
    md_file:write("- **" .. file .. "**: " .. #matches .. " matches\n")
end
md_file:write("\n")

-- Write detailed information for each file
md_file:write("## Detailed Matches\n\n")

for file, matches in pairs(grouped_results) do
    -- Section header for each file
    md_file:write("### File: `" .. file .. "`\n")
    md_file:write("Total Matches: **" .. #matches .. "**\n\n")

    -- Write matches in a table format for better readability
    md_file:write("| Line Number | Match Content |\n")
    md_file:write("|-------------|---------------|\n")

    for _, match in ipairs(matches) do
        local line_number = match.line_number
        local line_content = match.line_content:gsub("|", "\\|")  -- Escape pipe characters to prevent breaking table formatting
        md_file:write("| " .. line_number .. " | `" .. line_content .. "` |\n")
    end

    md_file:write("\n")
end

-- Provide some general information at the end of the report
md_file:write("---\n")
md_file:write("### Notes\n")
md_file:write("- Matches are highlighted using inline code blocks for easier identification.\n")
md_file:write("- Please verify the matches for accuracy, as some patterns may result in false positives.\n\n")

md_file:write("**End of Report**\n")

md_file:close()
print("Advanced Markdown report generated: search_report.md")
