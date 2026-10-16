# Spec Delta

## Purpose

The single-page UI of the browser agent app: writing the capability spec delta for the prompt/session interface and API-key setup.

## ADDED Requirements

### Requirement: Prompt entry
The UI SHALL present a text input that submits the entered prompt as new input to the agent conversation.

#### Scenario: Submit a prompt
- **WHEN** the user types text into the prompt field and submits it
- **THEN** the prompt is admitted to the conversation as an input message and appears in the transcript

#### Scenario: Empty prompt
- **WHEN** the user submits while the prompt field is empty
- **THEN** no message is admitted and the UI indicates the field is required

### Requirement: Session transcript display
The UI SHALL render all conversation entries from the durable session, including the user's prompts and the agent's assistant messages, in submission order.

#### Scenario: Fresh session
- **WHEN** the app loads with an existing stored session
- **THEN** the previously committed transcript entries are rendered below the prompt field

#### Scenario: Entries appear in order
- **WHEN** the user exchanges several prompts with the agent
- **THEN** prompts and answers are rendered in the order they were committed

### Requirement: Live streaming display
The UI SHALL show the in-progress agent response, including streamed partial content, while the agent is generating.

#### Scenario: Response streams in
- **WHEN** the agent is generating a response
- **THEN** partial streamed content is displayed as it arrives until the response is committed

### Requirement: Tool-call progress display
The UI SHALL show the current state of tool calls (running with their arguments, then completed with their output or error).

#### Scenario: Tool call completes
- **WHEN** the agent calls the weather tool and it finishes
- **THEN** the tool call's name, arguments, result, and duration are visible in the session view

### Requirement: API key setup
The UI SHALL provide a screen where the user can store the model gateway API key, and SHALL block prompting until a stored key can be used for model access.

#### Scenario: First use without key
- **WHEN** the app loads and no API key has been stored
- **THEN** the key-setup screen is shown and the prompt field is not active

#### Scenario: Save an API key
- **WHEN** the user enters an API key on the setup screen and confirms
- **THEN** the key is stored encrypted and prompting becomes available

#### Scenario: Invalid key used
- **WHEN** the user submits a prompt and the stored API key is rejected by the gateway
- **THEN** the UI surfaces the authentication failure and offers re-entry of the key
