use crate::search::result::SearchMatch;
use rlua::{Function, Lua, Result as LuaResult, RluaCompat, Table};
use std::fs;

pub struct LuaPlugin {
    pub lua: Lua,
}

impl LuaPlugin {
    pub fn new() -> Self {
        LuaPlugin { lua: Lua::new() }
    }

    /// Load and execute the Lua script, providing the search results.
    pub fn run_script(&self, script_path: &str, results: &[SearchMatch]) -> LuaResult<()> {
        let script_content = fs::read_to_string(script_path)?;
        let lua_ctx = &self.lua;
        let lua_results_table = lua_ctx.create_table()?;
        for (i, result) in results.iter().enumerate() {
            let result_table = Self::create_result_table(lua_ctx, result)?;
            lua_results_table.set(i + 1, result_table)?;
        }

        lua_ctx.globals().set("results", lua_results_table)?;
        lua_ctx.load(&script_content).exec()?;
        Ok(())
    }

    /// Execute a callback Lua function for each search result
    pub fn execute_callback(&self, callback_name: &str, results: &[SearchMatch]) -> LuaResult<()> {
        let lua_ctx = &self.lua;
        let callback: rlua::Function = lua_ctx.globals().get(callback_name)?;

        for result in results {
            let result_table = Self::create_result_table(lua_ctx, result)?;
            callback.call::<_, ()>(result_table)?;
        }

        Ok(())
    }

    /// Check if the Lua context has a specific global function.
    pub fn has_function(&self, func_name: &str) -> LuaResult<bool> {
        let lua_ctx = &self.lua;
        let value = lua_ctx.globals().get::<_, Option<Function>>(func_name)?;
        Ok(value.is_some())
    }

    /// Helper function to convert search results into a Lua table.
    pub fn create_result_table<'lua>(
        lua_ctx: rlua::Context<'lua>,
        result: &SearchMatch,
    ) -> LuaResult<Table<'lua>> {
        let lua_result = lua_ctx.create_table()?;

        // Set the fields explicitly and ensure they are valid.
        lua_result.set("file", result.file.clone())?;
        lua_result.set("line_number", result.line_number)?;
        lua_result.set("line_content", result.line_content.clone())?;
        let matches_table = lua_ctx.create_table()?;

        for (i, &(start, end)) in result.matches.iter().enumerate() {
            let match_table = lua_ctx.create_table()?;
            match_table.set("start", start)?;
            match_table.set("end", end)?;
            matches_table.set(i + 1, match_table)?;
        }

        Ok(lua_result)
    }
}
