# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Extab is a lightweight (~10MB), privacy-first AI assistant desktop application built with Tauri (Rust backend) and React + TypeScript (frontend). It's designed to be stealthy, translucent, and always accessible via global shortcuts. The app supports multiple AI providers, speech-to-text, system audio capture, screenshot capture, and conversation history management.

**Key Architecture Principle**: Direct frontend-to-AI API calls (no backend proxy) for maximum privacy. API keys are stored in browser localStorage, never sent to any server.

## Development Commands

### Build & Run
```bash
# Development mode (hot reload)
npm run tauri dev

# Build frontend only
npm run build

# Production build (creates installers in src-tauri/target/release/bundle/)
npm run tauri build

# Type checking
npm run type-check
```

### Platform-specific outputs
- macOS: `.dmg` files
- Windows: `.msi`, `.exe` installers
- Linux: `.deb`, `.rpm`, `.AppImage`

## Core Architecture

### Frontend (React + TypeScript)

#### Application Context (`src/contexts/app.context.tsx`)
Central state management using React Context. Manages:
- AI provider selection and configuration (both built-in and custom)
- STT provider selection and configuration
- System prompt and screenshot configuration
- Customizable UI settings (app icon visibility, always-on-top, element titles)
- Extab API license state
- **Critical**: Uses `curl2Json` to parse and validate custom provider curl commands
- Syncs all settings to localStorage with `STORAGE_KEYS` constants

#### Completion Hook (`src/hooks/useCompletion.ts`)
Main hook for AI interactions:
- Manages chat state: input, response, conversation history, attached files
- Handles streaming AI responses via `fetchAIResponse()` generator function
- Supports both standard text input and voice input via VAD
- Screenshot handling in two modes:
  - **Manual**: Adds screenshots to attachedFiles for user to submit with prompt
  - **Auto**: Immediately sends screenshot to AI with predefined prompt
- Conversation management: saves/loads/deletes chat history to localStorage
- Auto-scrolling, file attachments (max 6), image pasting
- Window resize coordination based on popover/mic/history states

#### AI Response Handling (`src/lib/functions/ai-response.function.ts`)
- `fetchAIResponse()`: Primary async generator for streaming AI responses
- `fetchExtabAIResponse()`: Uses Tauri IPC (`invoke("chat_stream")`) for Extab API
- Custom provider support via curl command parsing and dynamic variable replacement
- Handles both streaming and non-streaming responses
- Image support: converts base64 images to provider-specific format

#### Provider Configuration
- **AI Providers**: Defined in `src/config/ai-providers.constants.ts`
- **STT Providers**: Defined in `src/config/stt.constants.ts`
- **Custom Providers**: Users can add any provider via curl commands
- **Dynamic Variables**: `{{TEXT}}`, `{{IMAGE}}`, `{{SYSTEM_PROMPT}}`, `{{MODEL}}`, `{{API_KEY}}`
- Variable replacement happens in `deepVariableReplacer()` function

### Backend (Tauri + Rust)

#### Entry Points
- `src-tauri/src/main.rs`: Simple entry point that calls `extab_lib::run()`
- `src-tauri/src/lib.rs`: Core Tauri setup with plugin initialization and IPC command registration

#### Key Tauri Commands (IPC)
- `get_app_version()`: Returns app version from Cargo.toml
- `set_window_height(height: u32)`: Resizes window and repositions to top-center
- `capture_to_base64()`: Captures primary monitor screenshot using xcap, returns base64 PNG
- `chat_stream()`: Handles streaming AI responses for Extab API (emits `chat_stream_chunk` events)
- `transcribe_audio()`: Handles STT for Extab API
- `activate_license_api()`: License activation for Extab API
- `secure_storage_save/get/remove()`: Secure keychain storage for API keys
- `start/stop_system_audio_capture()`: Platform-specific system audio capture

#### Window Management (`src-tauri/src/window.rs`)
- `position_window_top_center()`: Positions window 54px from top, horizontally centered
- Window is configured in `tauri.conf.json`:
  - Default size: 700x54px
  - Transparent, no decorations, always on top (configurable)
  - `skipTaskbar: true`, `visibleOnAllWorkspaces: true`

#### Global Shortcuts (`src-tauri/src/shortcuts.rs`)
- Toggle Window: `Cmd+\` (macOS) / `Ctrl+\` (Windows/Linux)
- Voice Input: `Cmd+Shift+A` / `Ctrl+Shift+A`
- Screenshot: `Cmd+Shift+S` / `Ctrl+Shift+S`
- System Audio: `Cmd+Shift+M` / `Ctrl+Shift+M`
- Shortcuts registered on startup and checked for conflicts

#### System Audio Capture (`src-tauri/src/speaker/`)
Platform-specific implementations:
- **macOS**: `macos.rs` - Uses Cidre framework for audio capture
- **Windows**: `windows.rs` - Uses WASAPI
- **Linux**: `linux.rs` - Uses PulseAudio
- Streams f32 audio samples, converted to WAV format in commands.rs
- Audio data accumulated in ring buffer, sent to frontend via events

#### License & API Integration (`src-tauri/src/activate.rs` & `src-tauri/src/api.rs`)
- Extab API license validation and checkout URL generation
- `chat_stream()`: Streaming chat API calls with Server-Sent Events (SSE)
- `transcribe_audio()`: Audio transcription API for STT
- `fetch_models()`: Fetch available AI models from Extab API

### State Management & Storage

#### localStorage Keys (`src/config/constants.ts`)
All user data stored in browser localStorage:
- `curl_custom_ai_providers`: Custom AI provider configs (JSON array)
- `curl_selected_ai_provider`: Currently selected AI provider + variables
- `curl_custom_speech_providers`: Custom STT provider configs
- `curl_selected_stt_provider`: Currently selected STT provider + variables
- `chat_history`: Conversation history (array of ChatConversation objects)
- `system_prompt`: Custom system prompt (default: "You are a helpful AI assistant...")
- `screenshot_config`: Screenshot mode (manual/auto) + autoPrompt
- `system_audio_context`: System audio capture mode context
- `system_audio_quick_actions`: Quick action buttons for system audio
- `customizable`: UI customization settings (app icon, always-on-top, titles)
- `extab_api_enabled`: Whether to use Extab API instead of custom provider

#### Conversation History Structure
```typescript
interface ChatConversation {
  id: string;
  title: string; // Auto-generated from first 6 words of initial message
  messages: ChatMessage[];
  createdAt: number;
  updatedAt: number;
}

interface ChatMessage {
  id: string;
  role: "user" | "assistant" | "system";
  content: string;
  timestamp: number;
}
```

## Important Patterns

### Custom Provider System
1. User provides curl command in settings
2. `curl2Json` parses curl into JSON structure with URL, headers, body
3. Extract dynamic variables from curl (anything in `{{VAR_NAME}}` format)
4. Store provider config with curl string and parsed JSON in localStorage
5. At runtime, replace variables with actual values using `deepVariableReplacer()`
6. Make fetch request to provider API with processed config

### Streaming Response Pattern
```typescript
for await (const chunk of fetchAIResponse({ ... })) {
  // Yield chunks as they arrive
  fullResponse += chunk;
  setState(prev => ({ ...prev, response: prev.response + chunk }));
}
```

### Window Resizing
- Input-only state: 54px height
- Popover open (response/error): Dynamically resize based on content
- Mic open: Expand for voice input UI
- Message history open: Expand for history panel
- Files popover open: Expand for file preview
- `useWindowResize` hook manages all resize logic via `set_window_height` command

### Screenshot Flow
1. User presses `Cmd+Shift+S` or clicks screenshot button
2. `invoke("capture_to_base64")` captures primary monitor
3. Check `screenshotConfiguration.mode`:
   - **Manual**: Add to `attachedFiles[]`, user submits with custom prompt
   - **Auto**: Immediately call `handleScreenshotSubmit(base64, autoPrompt)`
4. Auto mode skips file attachment, directly streams AI response

### Voice Activity Detection (VAD)
- Uses `@ricky0123/vad-react` for voice activity detection
- When VAD detects speech end, audio buffer sent to STT provider
- STT transcription returned, passed to `submit(speechText)` function
- Voice input can be standalone or combined with text/images

## Testing

No test suite currently exists in the project. Tests in `node_modules` are from dependencies only.

## Platform-Specific Notes

### macOS
- Requires microphone and screen recording permissions (requested at runtime)
- Uses `ActivationPolicy::Accessory` for dock icon hiding
- Cidre framework for system audio capture
- `info.plist` embedded for bundle configuration

### Windows
- Uses `skip_taskbar` for app icon hiding
- WASAPI for system audio capture
- MSI and EXE installers generated

### Linux
- Uses `skip_taskbar` for app icon hiding
- PulseAudio for system audio capture
- `.desktop` file: `src-tauri/extab.desktop`
- Multiple package formats: `.deb`, `.rpm`, `.AppImage`

## Security & Privacy

- **No backend server**: Frontend makes direct API calls to AI providers
- **No telemetry**: No analytics or tracking
- **Local storage only**: API keys in localStorage, conversations in localStorage
- **Secure keychain**: Extab license keys stored in platform keychain (via tauri-plugin-keychain)
- **HTTPS only**: All API requests use HTTPS via `@tauri-apps/plugin-http`
- **Content security**: Markdown rendering uses `rehype-sanitize` to prevent XSS

## Common Development Tasks

### Adding a New Tauri Command
1. Add function in `src-tauri/src/lib.rs` or appropriate module
2. Add to `invoke_handler![]` macro in `lib.rs`
3. Call from frontend via `invoke("command_name", { args })`

### Modifying Window Behavior
- Edit `src-tauri/tauri.conf.json` for initial window config
- Edit `src-tauri/src/window.rs` for positioning logic
- Edit `src/hooks/useWindow.ts` for frontend window control

### Changing Global Shortcuts
- Edit `src-tauri/src/shortcuts.rs` for shortcut registration
- Platform-specific key mappings defined in `get_shortcuts()` function

### Debugging Streaming Responses
- Check browser Network tab for fetch requests (custom providers)
- Check Tauri console output for Rust command logs
- Frontend streaming handled by async generators - use `console.log` in loop

### Working with System Audio
- Platform-specific code in `src-tauri/src/speaker/{macos,windows,linux}.rs`
- Audio format: f32 samples at platform-specific sample rate
- Conversion to WAV happens in `src-tauri/src/speaker/commands.rs`
- Frontend listens for `system_audio_chunk` events

## Contributing Guidelines

**Acceptable PRs**: Bug fixes and feature improvements to existing functionality
**NOT Accepted**: PRs adding new AI providers or STT providers (use custom provider system instead)
