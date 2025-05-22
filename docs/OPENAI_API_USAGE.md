# OpenAI API Usage in codex-cli

This document outlines how the `codex-cli` utilizes the OpenAI API for its functionalities.

## 1. OpenAI Client Initialization (`codex-cli/src/utils/openai-client.ts`)

The OpenAI client is initialized and configured by the `createOpenAIClient` function. This function handles the setup for both standard OpenAI and Azure OpenAI services.

### Configuration:

-   **API Keys**:
    -   For standard OpenAI, the API key is retrieved from `config.openai.key`.
    -   For Azure OpenAI, the API key is retrieved from `config.azureOpenai.key`.
-   **Base URLs**:
    -   The base URL for standard OpenAI can be overridden using `config.openai.baseUrl`.
    -   For Azure OpenAI, the endpoint is taken from `config.azureOpenai.endpoint`.
-   **Other Configurations**:
    -   `OPENAI_ORGANIZATION`: This environment variable can be used to specify the OpenAI organization.
    -   `OPENAI_PROJECT`: This environment variable can be used to specify the OpenAI project.
    -   The client is instantiated with these configurations, along with default headers.
    -   `dangerouslyAllowBrowser`: This option is explicitly set to `true`, indicating usage in a non-browser environment where such an allowance is necessary.

The function returns an `OpenAI` client instance configured to interact with the chosen service.

## 2. Agent Loop (`codex-cli/src/utils/agent/agent-loop.ts`)

The `AgentLoop` class is central to managing interactions with the OpenAI API. It encapsulates the logic for sending requests, processing responses, and handling tool usage.

### Role:

-   It uses an `OpenAI` client instance (obtained from `createOpenAIClient`) to communicate with the API.
-   It manages the conversation flow, including maintaining context and handling streaming responses.

### Constructor Parameters:

The `AgentLoop` constructor accepts several key parameters:

-   `model`: The specific OpenAI model to be used (e.g., "gpt-4").
-   `provider`: Specifies whether to use "openai" or "azure" services.
-   `instructions`: The system prompt or specific instructions that guide the agent's behavior.
-   `config`: The application's configuration object, providing access to API keys and other settings.
-   `responseFormat`: (Optional) Specifies the desired response format (e.g., JSON).
-   `disableResponseStorage`: (Optional) A boolean that, if true, prevents OpenAI from storing the response for conversation context. This also means `previous_response_id` will not be used.
-   `enableReasoning`: (Optional) A boolean to enable reasoning steps from the model.

## 3. API Request Process

The `run` method in `AgentLoop` is responsible for constructing and sending requests to the OpenAI API.

### Request Construction:

-   The method takes the user's `input` (which includes the conversation history) and constructs the request payload.
-   It combines the system `instructions` (prepended with a `prefix` defining the agent's capabilities) with the user input.

### API Endpoints:

-   For standard OpenAI, it targets the `openai.responses.create` endpoint.
-   For Azure OpenAI, it uses a helper function `responsesCreateViaChatCompletions` which likely adapts the request for Azure's chat completions endpoint.

### Main Request Parameters:

-   `model`: The OpenAI model specified during `AgentLoop` initialization.
-   `instructions`: The system prompt, combined with the `prefix`.
-   `input`: An array of messages representing the conversation history, including the latest user prompt.
-   `tools`: An array of tool definitions available to the model. This includes:
    -   `shellFunctionTool`: A standard tool for executing shell commands.
    -   `localShellTool`: Another tool potentially for local shell execution, differentiated by its handling (typically for `codex` models).
    -   **MCP Tools**: Dynamically added based on active MCP server presets configured by the user. These tools have `type: "mcp"` and include `server_label` and `server_url` fields, pointing to external MCP-compliant servers. The `AgentLoop` fetches active presets via `codex-rs-cli config mcp list` and constructs these tool entries.
-   `tool_choice`: Configured as "auto", allowing the model to decide when to use a tool.
-   `stream`: Set to `true`, indicating that responses should be streamed.
-   `store`: This parameter is implicitly managed. If `disableResponseStorage` is `false` (the default), OpenAI is expected to store the response, and a `previous_response_id` is sent with subsequent requests to maintain context. If `disableResponseStorage` is `true`, this mechanism is bypassed.

## 4. Response Handling

The `AgentLoop` is designed to handle streaming responses from the OpenAI API.

### Streaming:

-   The `run` method iterates over the streamed response chunks.
-   Different event types from the stream trigger specific handling logic:
    -   `response.output_item.done`: Indicates the completion of a specific output item (e.g., a function call).
    -   `response.completed`: Signals the end of the entire response stream.

### Processing Response Item Types:

-   **`message`**: Regular text responses from the model are typically printed to the console.
-   **`function_call` / `local_shell_call`**:
    -   When the model decides to use a tool, it emits a `function_call` (for `shellFunctionTool`) or `local_shell_call` (for `localShellTool`).
    -   The `AgentLoop` handles these by:
        -   Parsing the tool name and arguments.
        -   Executing the command using `handleExecCommand` (for `shellFunctionTool`) or `handleLocalShellCall` (for `localShellTool`). These methods interact with the local shell.
        -   The output of the tool execution is then sent back to the model in a subsequent request to inform its next steps.
-   **`reasoning`**: If `enableReasoning` is true, these steps from the model are captured and can be displayed, providing insight into the model's thought process.

### Context Management:

-   **`lastResponseId`**: When `disableResponseStorage` is `false`, the `id` from the last response (`lastResponseId`) is stored and sent as `previous_response_id` in the next request. This allows OpenAI to link conversation turns.
-   **Local Transcript**: When `disableResponseStorage` is `true`, a local `transcript` array is maintained within the `AgentLoop`. This transcript accumulates user messages, assistant responses, and tool interactions to provide context for subsequent API calls, as OpenAI is not storing the history.

## 5. System Prompt

A crucial part of guiding the agent's behavior is the system prompt, which is defined by the `prefix` constant in `codex-cli/src/utils/agent/agent-loop.ts`.

### Role and Content of `prefix`:

-   **Defines Agent Persona**: It instructs the model to act as an AI programming assistant.
-   **Specifies Capabilities**:
    -   Code generation, explanation, and modification.
    -   Answering questions about code.
    -   Shell command execution via provided tools.
-   **Sets Constraints and Guidelines**:
    -   Emphasizes accuracy and conciseness.
    -   Instructs on how to format code blocks (using Markdown).
    -   Defines how to use the shell tool (e.g., asking for permission for destructive commands, using `stdin` for piped input).
    -   Advises on handling errors and providing context.
    -   Sets a knowledge cut-off and encourages the use of tools for current information.
    -   Includes a section on "IMPORTANT LESSONS" that refine its interaction style and tool usage based on learned best practices.
-   **Tool Usage Instructions**: Provides detailed instructions on when and how to use the available shell tools, including how to interpret user requests for commands.

This `prefix` is prepended to the user-provided `instructions` to form the complete system prompt sent to the OpenAI API.

## 6. Error Handling and Retries

The `AgentLoop` includes mechanisms to handle potential errors from the OpenAI API and to retry requests.

-   **Error Catching**: The code uses `try...catch` blocks to manage errors during API calls.
-   **Retry Logic**: While the provided snippets don't explicitly detail a sophisticated retry mechanism with backoff, typical HTTP clients (like the one `OpenAI` uses) often have built-in retries for transient network issues. The `AgentLoop` itself might implement additional retry logic for specific API errors such as:
    -   Rate limit errors (429).
    -   Server errors (5xx).
    -   Timeouts.
-   The `handleExecCommand` and `handleLocalShellCall` also include their own error handling for issues arising from shell command execution.

This structured approach to API interaction, response handling, and error management allows `codex-cli` to effectively leverage OpenAI's capabilities.

## 7. OpenAI NPM Package Documentation

This section details the `openai` npm package (version 4.x or higher, as used by `codex-cli`) and its relevance to the project. The information is sourced from the official NPM page and the `api.md` documentation from the package's GitHub repository.

### Overview of the `openai` NPM Package:

-   The `openai` package is the official Node.js library provided by OpenAI for interacting with their REST API. It offers a convenient way to access OpenAI functionalities from TypeScript or JavaScript.
-   The library is automatically generated from OpenAI's OpenAPI specification, ensuring it stays current with API changes.

### Key API Methods Used in `codex-cli`:

-   **`client.responses.create(params)`**:
    -   This is the primary method `codex-cli` uses for direct interactions with the OpenAI API when the configured `provider` is 'openai'.
    -   Key parameters utilized by `codex-cli` include:
        -   `model` (string): Specifies the ID of the model to use (e.g., `gpt-4o`).
        -   `instructions` (string): The system-level instructions that guide the model's behavior. In `codex-cli`, this is combined with a predefined `prefix`.
        -   `input` (array of `ResponseInputItem`): Represents the conversation history and the latest user prompt. Each item can be text, image, audio, etc.
        -   `stream` (boolean): When `true` (as used in `codex-cli`), the API streams back partial progress via Server-Sent Events.
        -   `tools` (array of `Tool`): Defines any tools the model can use. This includes standard tools like `shellFunctionTool`, `localShellTool`, and dynamically added **MCP Tools** (see above).
        -   `tool_choice` (string or object): Controls how the model selects tools. `codex-cli` uses "auto" to let the model decide.
        -   `store` (boolean, default `false`): If `true`, OpenAI will store the response for use in subsequent turns. `codex-cli` manages this via `disableResponseStorage` which, if `false`, implies `store` is effectively enabled, and `previous_response_id` is used to link conversation turns. If `disableResponseStorage` is `true`, then `store` is not used and `previous_response_id` is not sent.
        -   `previous_response_id` (string, optional): The ID of the previous response, used to maintain conversation context if `store` was used for that previous response.
-   **`client.chat.completions.create(params)`**:
    -   `codex-cli` utilizes this method when the `provider` is set to 'azure'. This is typically done through a helper function `responsesCreateViaChatCompletions` (found in `codex-cli/src/utils/responses.ts`), which adapts the request for Azure OpenAI Service, as it often aligns with the chat completions API.
    -   The structure is similar to `responses.create`, but with some key differences in parameters to match the chat completions format:
        -   `model` (string): Specifies the deployment ID of the model on Azure.
        -   `messages` (array of `ChatCompletionMessageParam`): An array of message objects representing the conversation history. This typically includes messages with roles like `system`, `user`, and `assistant`, replacing the separate `instructions` and `input` fields of the `responses.create` API.
        -   `stream` (boolean): Also set to `true` in `codex-cli` for streaming responses.
        -   `tools` (array of `ChatCompletionTool`): Similar to the `responses` API, defines tools available to the model, including standard and **MCP Tools**.
        -   `tool_choice` (string or object): Similar to the `responses` API, controls tool selection.

### Streaming:

-   The `openai` package fully supports streaming responses by setting the `stream: true` parameter in API calls like `responses.create` or `chat.completions.create`.
-   `codex-cli` heavily relies on this feature to provide real-time feedback to the user. It processes various events from the stream. The `api.md` (under `Responses` and `Beta > Realtime`) lists various `ResponseStreamEvent` types such as `ResponseOutputItemAddedEvent`, `ResponseTextDeltaEvent`, `ResponseFunctionCallArgumentsDeltaEvent`, and `ResponseCompletedEvent`, which `codex-cli` listens to for updating the display, handling tool calls, and managing the conversation flow.

### Types:

-   The `api.md` file, included with the `openai` package, provides an extensive list of TypeScript types for all request parameters, response payloads, and nested objects. Examples relevant to `codex-cli` include:
    -   `Response`, `ResponseInputItem`, `ResponseOutputItem`
    -   `Tool`, `FunctionTool`, `ComputerTool`, and potentially custom types for MCP tools if defined (though often they are generic objects with a `type: "mcp"` field).
    -   `ResponseStreamEvent` and its various subtypes (e.g., `ResponseTextDeltaEvent`, `ResponseOutputItemDoneEvent`)
    -   `ChatCompletion`, `ChatCompletionMessageParam`, `ChatCompletionTool`
-   `codex-cli` leverages these TypeScript types extensively to ensure type safety, facilitate development with auto-completion, and maintain robust interactions with the OpenAI API.

### Error Handling and Other Features:

-   The `openai` library includes built-in mechanisms for handling API errors. It throws specific error classes (e.g., `APIError`, `BadRequestError`, `RateLimitError`) based on the HTTP status codes and API responses.
-   The package also supports automatic retries for certain types of errors (e.g., network issues, rate limits, server errors) with exponential backoff, configurable via `maxRetries`.
-   Timeouts for requests can also be configured using the `timeout` option.
-   `codex-cli` utilizes these features as part of its own error handling and retry logic within the `AgentLoop` to manage API interactions more resiliently.
