-- Lua script to generate an HTML report of matches
local html_file = io.open("search_report.html", "w")

-- Write HTML header
html_file:write("<html><head><title>Search Report</title></head><body>\n")
html_file:write("<h1>Search Matches Report</h1>\n")
html_file:write("<ul>\n")

-- Write each search match as an HTML list item
for i = 1, #results do
    local result = results[i]
    html_file:write(string.format(
        "<li><strong>File:</strong> %s, <strong>Line:</strong> %d, <strong>Content:</strong> %s</li>\n",
        result.file, result.line_number, result.line_content
    ))
end

html_file:write("</ul>\n")
html_file:write("</body></html>\n")

html_file:close()
print("HTML report generated: search_report.html")
