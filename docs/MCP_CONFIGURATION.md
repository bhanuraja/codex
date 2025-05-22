# Model-Controlled Programming (MCP) Configuration

The `codex-cli` supports Model-Controlled Programming (MCP), allowing the underlying AI model to interact with external tools and services. This is achieved by configuring MCP server presets, which inform the model about available external capabilities.

## Configuring MCP Server Presets

You can manage MCP server presets using slash commands directly within the `codex-cli` chat interface. These commands interact with your local `~/.codex/config.toml` file (or `$CODEX_HOME/config.toml` if the environment variable is set).

Here are the available `/mcp` commands:

*   **`/mcp list`**
    *   **Description:** Lists all currently configured MCP server presets, showing their label, URL, and whether they are active ("Enabled") or inactive ("Disabled").
    *   **Example Output:**
        ```
        MCP Presets:
        Label    | URL                        | Status
        ---------|----------------------------|---------
        my_api   | http://localhost:8080/mcp  | Enabled
        another  | http://example.com/api     | Disabled
        ```

*   **`/mcp add <label> <url>`**
    *   **Description:** Adds a new MCP server preset with the given unique `label` and `url`. If a preset with the same `label` already exists, its URL will be updated, and it will be enabled. New presets are enabled by default.
    *   **Example:** `/mcp add my_api http://localhost:8080/mcp`

*   **`/mcp remove <label>`**
    *   **Description:** Removes the MCP server preset identified by `<label>`.
    *   **Example:** `/mcp remove my_api`

*   **`/mcp enable <label>`**
    *   **Description:** Activates the MCP server preset identified by `<label>`. Active presets are included in the list of tools sent to the OpenAI model during API calls.
    *   **Example:** `/mcp enable my_api`

*   **`/mcp disable <label>`**
    *   **Description:** Deactivates the MCP server preset identified by `<label>`. Inactive presets are not sent to the OpenAI model.
    *   **Example:** `/mcp disable my_api`

*   **`/mcp help`**
    *   **Description:** Displays a help message listing all available `/mcp` commands and their descriptions.

## How MCP Presets are Used

When you issue a prompt to `codex-cli`, the system gathers all *active* (enabled) MCP server presets. For each active preset, a special tool object is constructed and included in the `tools` array sent to the OpenAI API. This allows the model to become aware of these external servers and the tools they might offer.

The structure of an MCP tool entry sent to OpenAI is as follows:

```json
{
  "type": "mcp",
  "server_label": "your_preset_label",
  "server_url": "your_preset_url"
}
```

The model can then decide if any of its tasks can be aided by invoking a tool potentially hosted on one of these MCP servers. If it chooses to use an MCP tool, it will typically first request a list of available tools from the MCP server (e.g., via a `tools/list` request to the `server_url`) and then make a specific tool call (e.g., `tools/call`).

## Internal `codex-rs` MCP Server

`codex-cli` is designed to work in conjunction with `codex-rs`, which includes an internal MCP server component. This internal server can expose `codex-rs`'s own capabilities (like advanced file system operations, code analysis, etc.) as tools to the OpenAI model.

*   **Activation:** The `codex-rs` MCP server is typically started automatically when `codex-cli` runs, or it can be run as a separate process (`codex-rs-cli mcp`). The specifics of its activation and port may depend on the `codex-rs` configuration.
*   **Default Configuration:** When the internal `codex-rs` MCP server is active and its details are known to `codex-cli` (this part is currently under development for full automation), it will be included in the tools list sent to OpenAI.
    *   A common convention for its label might be `"codex_internal"`.
    *   The URL will typically be `http://localhost:PORT`, where `PORT` is the port the `codex-rs` MCP server is listening on (e.g., often a default like `3031` or dynamically assigned).
*   **Usage:** You can also manually add the internal server as a preset if you know its running URL, for example: `/mcp add codex_internal http://localhost:3031` (assuming 3031 is the port). However, future versions aim to make this detection and inclusion more seamless.

By leveraging both external and internal MCP servers, `codex-cli` can significantly extend the range of tasks the AI model can perform.

---

## Manual Testing Steps for MCP Integration

This section outlines steps for manually testing the MCP server preset configuration and its integration with OpenAI API calls.

### A. Testing MCP Preset Configuration Commands:

1.  Start `codex-cli`.
2.  Use `/mcp list`.
    *   *Expected:* "No MCP presets configured." or an empty table.
3.  Use `/mcp add test_server http://localhost:1234/mcp`.
    *   *Expected:* "MCP preset 'test_server' added/updated successfully."
4.  Use `/mcp list` again.
    *   *Expected:* Table shows `test_server` with URL `http://localhost:1234/mcp` and status "Enabled".
5.  Use `/mcp disable test_server`.
    *   *Expected:* "MCP preset 'test_server' disabled successfully."
6.  Use `/mcp list`.
    *   *Expected:* Table shows `test_server` with status "Disabled".
7.  Use `/mcp enable test_server`.
    *   *Expected:* "MCP preset 'test_server' enabled successfully."
8.  Use `/mcp list`.
    *   *Expected:* Table shows `test_server` with status "Enabled".
9.  Use `/mcp add test_server http://localhost:5678/mcp_updated` to update its URL.
    *   *Expected:* "MCP preset 'test_server' added/updated successfully."
10. Use `/mcp list`.
    *   *Expected:* Table shows `test_server` with URL `http://localhost:5678/mcp_updated` and status "Enabled".
11. Use `/mcp remove test_server`.
    *   *Expected:* "MCP preset 'test_server' removed successfully."
12. Use `/mcp list`.
    *   *Expected:* "No MCP presets configured." or an empty table.
13. Test `/mcp help`.
    *   *Expected:* A help message listing all `/mcp` commands and their descriptions.

### B. Verifying MCP Presets in OpenAI API Calls:

1.  Start `codex-cli`.
2.  Add a known, *active* MCP preset: `/mcp add my_mcp http://example.com/api/mcp`.
3.  **Temporarily modify `codex-cli/src/utils/agent/agent-loop.ts`**:
    *   Locate the `AgentLoop#run` method.
    *   Just before the line `stream = await responseCall(...)`, add the following:
        ```typescript
        console.log("MCP Tools being sent to OpenAI:", JSON.stringify(tools.filter(t => t.type === 'mcp'), null, 2));
        ```
    *   *(Note: `tools` is the variable holding the array of all tools being sent. We filter for `type: 'mcp'` for clarity).*
4.  Rebuild and run `codex-cli` if necessary.
5.  Issue any command that triggers an OpenAI API call (e.g., "hello").
6.  Check the console output where `codex-cli` is running.
    *   *Expected:* You should see the `console.log` output, and it should include an entry like:
        ```json
        [
          {
            "type": "mcp",
            "server_label": "my_mcp",
            "server_url": "http://example.com/api/mcp"
          }
        ]
        ```
        (Other MCP tools might be present if more are active).
7.  **Important:** Remember to remove the temporary `console.log` from `agent-loop.ts` and rebuild after testing.

### C. End-to-End Test with Internal `codex-rs` MCP Server (Conceptual - requires setup):

This test verifies that `codex-cli` can communicate with the MCP server component of `codex-rs`.

1.  **Setup `codex-rs` MCP Server:**
    *   Ensure the `codex-rs` mcp-server component is configured to run. This might involve:
        *   Running `codex-rs-cli mcp` in a separate terminal.
        *   Or, ensuring `codex-cli` is set up to launch it automatically (if this feature exists).
    *   Note the host and port the `codex-rs` MCP server is listening on (e.g., `http://localhost:3031`).
2.  **Configure `codex-cli`:**
    *   If `codex-cli` does not automatically detect/add the internal server:
        *   Use `/mcp add codex_internal http://localhost:PORT` (replace `PORT` with the actual port).
    *   Ensure this preset is enabled.
3.  **Identify an Internal Tool:**
    *   Check the `codex-rs` codebase (e.g., `codex-rs/mcp-server/src/codex_tool_config.rs` or similar) to find a tool exposed by the internal server. For example, a tool named `file.read_file` might exist.
4.  **Prompt the Model:**
    *   In `codex-cli`, formulate a prompt that would logically require the model to use the identified internal tool.
    *   Example: "Read the first 2 lines of the file named `src/main.rs` using the `codex_internal` server's `file.read_file` tool." (The prompt might need to be less explicit about the server if the model is expected to discover it).
5.  **Observe:**
    *   **`codex-cli` logs/console:** Look for evidence of the OpenAI request (tools list should include `codex_internal`) and the subsequent response.
    *   **`codex-rs` mcp-server logs/console:**
        *   Check if it received a `tools/list` request.
        *   Check if it received a `tools/call` request for the specific tool (e.g., `file.read_file`).
        *   Observe the result it sent back.
    *   **`codex-cli` UI:** The final response from the model should ideally incorporate the result from the internal MCP tool (e.g., display the content of the file).

### D. End-to-End Test with an External Test MCP Server (Conceptual - requires setup):

This test verifies communication with any MCP-compliant third-party server.

1.  **Setup External MCP Server:**
    *   Deploy or identify a simple, publicly accessible (or locally accessible if `codex-cli` can reach it) MCP-compliant test server.
    *   This server should expose at least one basic tool (e.g., a tool that echoes input, or returns the current date/time).
    *   Ensure you know its URL and the exact name/schema of the tool(s) it exposes.
2.  **Configure `codex-cli`:**
    *   Use `/mcp add external_test <URL_of_test_server>` in `codex-cli`.
    *   Ensure this preset is enabled.
3.  **Prompt the Model:**
    *   Formulate a prompt that would lead the model to use a tool from the `external_test` server.
    *   Example: "Use the echo tool on the `external_test` server with the input 'hello world'."
4.  **Observe:**
    *   **External Test MCP Server logs:** Check for incoming requests (`tools/list`, `tools/call`).
    *   **`codex-cli` logs/console:** Inspect the OpenAI request and response.
    *   **`codex-cli` UI:** The model's response should reflect the output from the external tool.

These testing steps cover the configuration interface, the mechanism of sending MCP tools to OpenAI, and conceptual end-to-end flows.
