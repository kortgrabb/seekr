-- Lua script that generates a CSV report of matches
local csv_file = io.open("search_report.csv", "w")

-- Write CSV header
csv_file:write("File,Line Number,Content\n")

-- Write each search match as a line in the CSV
for i = 1, #results do
    local result = results[i]
    local escaped_content = result.line_content:gsub('"', '""')
    local csv_line = string.format('%s,%d,"%s"\n', result.file, result.line_number, escaped_content)
    csv_file:write(csv_line)
end

csv_file:close()
print("CSV report generated: search_report.csv")
