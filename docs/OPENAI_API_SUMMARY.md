## OpenAI API and `openai` NPM Package Usage in `codex-cli`

The `codex-cli` leverages the capabilities of the OpenAI API primarily through the official `openai` npm package (v4.x and higher). This package serves as the foundation for all interactions with OpenAI services.

**1. Client Initialization and Configuration:**

-   The `codex-cli` initializes an OpenAI client using the `createOpenAIClient` function located in `src/utils/openai-client.ts`.
-   This function supports configuration for both standard OpenAI services and Azure OpenAI services.
-   Key configurations include API keys, base URLs/endpoints, and organization/project IDs, which are sourced from the application's configuration and environment variables.
-   The `dangerouslyAllowBrowser` option is set to `true`, as `codex-cli` operates in a non-browser (Node.js) environment.

**2. Core API Interactions:**

The `AgentLoop` class (`src/utils/agent/agent-loop.ts`) is central to managing API requests and responses.

-   **Standard OpenAI**: For direct OpenAI interactions, `codex-cli` uses the `client.responses.create()` method. This method is used to send comprehensive requests that include the model ID, system instructions, conversation history (input), and definitions for available tools.
-   **Azure OpenAI**: When configured for Azure, `codex-cli` uses a wrapper function, `responsesCreateViaChatCompletions` (from `src/utils/responses.ts`). This function adapts the requests to the `client.chat.completions.create()` method, which is more aligned with Azure OpenAI's API structure. The `messages` parameter (containing system, user, and assistant roles) is used here instead of separate `instructions` and `input`.

**3. Key Features Utilized:**

-   **Streaming**: `codex-cli` makes extensive use of API streaming (`stream: true`). This allows for real-time processing of responses, providing immediate feedback to the user. The application handles various stream events to manage incoming data, tool calls, and conversation flow.
-   **Tool Usage (Function Calling)**: The OpenAI models are provided with definitions for tools (e.g., `shellFunctionTool`, `localShellTool`) that they can invoke. `codex-cli` sets `tool_choice: "auto"`, allowing the model to decide when to use these tools. The application then handles the execution of these tools and sends the results back to the model.
-   **System Prompts**: A detailed system prompt, defined as a `prefix` in `AgentLoop`, instructs the model on its persona (AI programming assistant), capabilities, constraints, and how to use tools effectively. This prompt is combined with user-provided instructions to guide the model's behavior.

**4. Conversation Context Management:**

`codex-cli` employs two main strategies for managing conversation context:

-   **API-Side Storage**: When `disableResponseStorage` is `false` (default), `codex-cli` leverages OpenAI's context management. The `id` from the last response (`lastResponseId`) is captured and sent as `previous_response_id` in subsequent requests. This allows OpenAI to link conversation turns.
-   **Local Transcript**: If `disableResponseStorage` is `true`, OpenAI does not store the response history. In this scenario, `codex-cli` maintains a local `transcript` array within the `AgentLoop`. This array accumulates user messages, assistant responses, and tool interactions, providing the necessary context for subsequent API calls.

This structured approach enables `codex-cli` to effectively utilize the OpenAI API for its code generation, explanation, and shell command execution functionalities, providing an interactive and responsive user experience.
