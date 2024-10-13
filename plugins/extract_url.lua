-- Lua script to extract URLs from search matches
local url_pattern = "https?://[%w-_%.%?%.:/%+=&]+"

print("Extracted URLs from matches:")

for i = 1, #results do
    local result = results[i]
    for url in string.gmatch(result.line_content, url_pattern) do
        print("File: " .. result.file .. ", Line: " .. result.line_number .. ", URL: " .. url)
    end
end
