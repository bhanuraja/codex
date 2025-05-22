// Defines the available slash commands and their descriptions.
// Used for autocompletion in the chat input.
export interface SlashCommand {
  command: string;
  description: string;
}

export const SLASH_COMMANDS: Array<SlashCommand> = [
  {
    command: "/clear",
    description: "Clear conversation history and free up context",
  },
  {
    command: "/clearhistory",
    description: "Clear command history",
  },
  {
    command: "/compact",
    description:
      "Clear conversation history but keep a summary in context. Optional: /compact [instructions for summarization]",
  },
  { command: "/history", description: "Open command history" },
  { command: "/sessions", description: "Browse previous sessions" },
  { command: "/help", description: "Show list of commands" },
  { command: "/model", description: "Open model selection panel" },
  { command: "/approval", description: "Open approval mode selection panel" },
  {
    command: "/bug",
    description: "Generate a prefilled GitHub issue URL with session log",
  },
  {
    command: "/diff",
    description:
      "Show git diff of the working directory (or applied patches if not in git)",
  },
  {
    command: "/mcp list",
    description: "Lists all configured MCP server presets.",
  },
  {
    command: "/mcp add <label> <url>",
    description:
      "Adds or updates an MCP server preset (e.g., /mcp add myapi http://localhost:8080/mcp).",
  },
  {
    command: "/mcp remove <label>",
    description: "Removes an MCP server preset.",
  },
  {
    command: "/mcp enable <label>",
    description: "Enables an MCP server preset to be used in OpenAI calls.",
  },
  {
    command: "/mcp disable <label>",
    description: "Disables an MCP server preset.",
  },
  {
    command: "/mcp help",
    description: "Shows help for MCP server preset commands.",
  },
];
